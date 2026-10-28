//! Historial y deshacer.
//!
//! El diario es un JSON Lines que solo crece. Cada lote se escribe asi:
//!   1. todas las entradas `pending` del lote en una escritura y un fsync
//!   2. se ejecutan los movimientos
//!   3. las entradas que salieron bien y las que fallaron se escriben por bloques
//!
//! Un fsync por archivo seria inviable con miles de archivos. Si el proceso muere
//! entre el paso 1 y el 3, la siguiente apertura marca esas entradas como fallidas.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::Error;
use crate::mover;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Pending,
    Done,
    Failed,
    /// El origen y el destino existen a la vez. No se toca ningun archivo:
    /// la persona decide que hacer despues de revisar el historial.
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Move,
    Copy,
    Trash,
    Undo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub v: u32,
    pub ts: String,
    pub batch: String,
    pub state: State,
    pub op: Op,
    pub from: PathBuf,
    #[serde(default)]
    pub to: Option<PathBuf>,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub category: String,
    /// Clave estable, no traducida, para explicar una entrada en revision.
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewEntry {
    pub from: PathBuf,
    pub to: Option<PathBuf>,
    pub size: u64,
    pub op: Op,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct BatchSummary {
    pub id: String,
    pub at: String,
    pub files: usize,
    pub bytes: u64,
    pub destinations: Vec<String>,
    pub trashed: usize,
    pub undone: bool,
    pub review: usize,
}

#[derive(Debug, Clone, Default)]
pub struct UndoReport {
    pub restored: usize,
    pub skipped: usize,
    pub skipped_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Totals {
    pub files: u64,
    pub bytes: u64,
    pub categories: Vec<(String, u64)>,
}

pub struct Journal {
    path: PathBuf,
    records: Vec<Record>,
}

impl Journal {
    pub fn open(path: PathBuf) -> Result<Self, Error> {
        let mut journal = Self {
            path,
            records: Vec::new(),
        };
        journal.load()?;
        journal.recover()?;
        Ok(journal)
    }

    fn load(&mut self) -> Result<(), Error> {
        if !self.path.exists() {
            return Ok(());
        }

        let file = File::open(&self.path).map_err(|err| Error::io(&self.path, err))?;
        for line in BufReader::new(file).lines() {
            let Ok(line) = line else { continue };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Una linea corrupta no invalida el resto del historial.
            if let Ok(record) = serde_json::from_str::<Record>(trimmed) {
                self.records.push(record);
            }
        }

        Ok(())
    }

    fn recover(&mut self) -> Result<(), Error> {
        let mut updates = Vec::new();

        for record in &self.records {
            if record.state != State::Pending {
                continue;
            }

            let resolved = self.records.iter().any(|other| {
                other.batch == record.batch
                    && other.from == record.from
                    && other.state != State::Pending
            });

            if resolved {
                continue;
            }

            let mut settled = record.clone();
            settled.ts = now_stamp();
            let (state, note) = settle(record);
            settled.state = state;
            settled.note = note;
            updates.push(settled);
        }

        if updates.is_empty() {
            return Ok(());
        }

        self.write(&updates)
    }

    /// Una escritura y un fsync para todo el bloque.
    fn write(&mut self, records: &[Record]) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        let mut buffer = String::new();
        for record in records {
            let line = serde_json::to_string(record)
                .map_err(|err| Error::journal(&self.path, err.to_string()))?;
            buffer.push_str(&line);
            buffer.push('\n');
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|err| Error::io(&self.path, err))?;

        file.write_all(buffer.as_bytes())
            .map_err(|err| Error::io(&self.path, err))?;
        file.sync_data().map_err(|err| Error::io(&self.path, err))?;

        self.records.extend_from_slice(records);
        Ok(())
    }

    pub fn begin(&mut self, batch: &str, entries: &[NewEntry]) -> Result<(), Error> {
        let ts = now_stamp();
        let records: Vec<Record> = entries
            .iter()
            .map(|entry| Record {
                v: 1,
                ts: ts.clone(),
                batch: batch.to_string(),
                state: State::Pending,
                op: entry.op,
                from: entry.from.clone(),
                to: entry.to.clone(),
                size: entry.size,
                category: entry.category.clone(),
                note: None,
            })
            .collect();

        self.write(&records)
    }

    pub fn mark(&mut self, batch: &str, entries: &[NewEntry], state: State) -> Result<(), Error> {
        let ts = now_stamp();
        let records: Vec<Record> = entries
            .iter()
            .map(|entry| Record {
                v: 1,
                ts: ts.clone(),
                batch: batch.to_string(),
                state,
                op: entry.op,
                from: entry.from.clone(),
                to: entry.to.clone(),
                size: entry.size,
                category: entry.category.clone(),
                note: None,
            })
            .collect();

        self.write(&records)
    }

    pub fn batch_summaries(&self) -> Vec<BatchSummary> {
        let mut order: Vec<String> = Vec::new();
        let mut summaries: std::collections::HashMap<String, BatchSummary> =
            std::collections::HashMap::new();

        for record in &self.records {
            if !matches!(record.state, State::Done | State::Review) {
                continue;
            }

            if !summaries.contains_key(&record.batch) {
                order.push(record.batch.clone());
                summaries.insert(
                    record.batch.clone(),
                    BatchSummary {
                        id: record.batch.clone(),
                        at: record.ts.clone(),
                        files: 0,
                        bytes: 0,
                        destinations: Vec::new(),
                        trashed: 0,
                        undone: false,
                        review: 0,
                    },
                );
            }

            let Some(summary) = summaries.get_mut(&record.batch) else {
                continue;
            };

            if record.state == State::Review {
                summary.review += 1;
                continue;
            }

            match record.op {
                Op::Move | Op::Copy => {
                    summary.files += 1;
                    summary.bytes += record.size;
                    if let Some(to) = &record.to {
                        if let Some(folder) = to.parent().and_then(|parent| parent.file_name()) {
                            let folder = folder.to_string_lossy().into_owned();
                            if !summary.destinations.contains(&folder) {
                                summary.destinations.push(folder);
                            }
                        }
                    }
                }
                Op::Trash => {
                    summary.files += 1;
                    summary.bytes += record.size;
                    summary.trashed += 1;
                }
                Op::Undo => {
                    summary.undone = true;
                }
            }
        }

        order
            .into_iter()
            .rev()
            .filter_map(|id| summaries.remove(&id))
            .collect()
    }

    pub fn last_batch(&self) -> Option<String> {
        self.batch_summaries()
            .into_iter()
            .find(|batch| !batch.undone && batch.files > 0)
            .map(|batch| batch.id)
    }

    pub fn undo(&mut self, batch: &str) -> Result<UndoReport, Error> {
        let undone: HashSet<PathBuf> = self
            .records
            .iter()
            .filter(|record| record.batch == batch && record.op == Op::Undo)
            .map(|record| record.from.clone())
            .collect();

        let mut report = UndoReport::default();
        let mut undo_records = Vec::new();

        // Los archivos enviados a la papelera se recuperan del registro real de
        // Windows. Si la papelera ya no los tiene, se avisa y no se toca nada.
        let mut trashed: Vec<PathBuf> = self
            .records
            .iter()
            .filter(|record| {
                record.batch == batch && record.state == State::Done && record.op == Op::Trash
            })
            .map(|record| record.from.clone())
            .filter(|path| !undone.contains(path))
            .collect();
        trashed.sort();
        trashed.dedup();

        if !trashed.is_empty() {
            #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
            {
                let wanted: HashSet<PathBuf> = trashed.iter().cloned().collect();
                match trash::os_limited::list() {
                    Ok(items) => {
                        let found: Vec<_> = items
                            .into_iter()
                            .filter(|item| wanted.contains(&item.original_path()))
                            .collect();
                        for path in &trashed {
                            if !found.iter().any(|item| item.original_path() == *path) {
                                report.skipped += 1;
                                report.skipped_paths.push(path.clone());
                            }
                        }
                        if !found.is_empty() {
                            let restored: Vec<PathBuf> =
                                found.iter().map(|item| item.original_path()).collect();
                            match trash::os_limited::restore_all(found) {
                                Ok(()) => {
                                    for path in restored {
                                        report.restored += 1;
                                        undo_records.push(Record {
                                            v: 1,
                                            ts: now_stamp(),
                                            batch: batch.to_string(),
                                            state: State::Done,
                                            op: Op::Undo,
                                            from: path,
                                            to: None,
                                            size: 0,
                                            category: "Duplicados".to_string(),
                                            note: None,
                                        });
                                    }
                                }
                                Err(_) => {
                                    for path in restored {
                                        report.skipped += 1;
                                        report.skipped_paths.push(path);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        for path in trashed {
                            report.skipped += 1;
                            report.skipped_paths.push(path);
                        }
                    }
                }
            }
        }

        let mut pending: Vec<Record> = self
            .records
            .iter()
            .filter(|record| {
                record.batch == batch
                    && record.state == State::Done
                    && matches!(record.op, Op::Move | Op::Copy)
                    && record.to.is_some()
            })
            .cloned()
            .collect();

        pending.reverse();

        for record in pending {
            let Some(current) = record.to.clone() else {
                continue;
            };

            if undone.contains(&current) {
                continue;
            }

            if !current.exists() {
                report.skipped += 1;
                report.skipped_paths.push(current);
                continue;
            }

            // Una copia se deshace retirando el duplicado. El original sigue en
            // su sitio, asi que no hay nada que devolver. La copia va a la
            // papelera para que la persona pueda recuperarla si la quiere.
            if record.op == Op::Copy && record.from.exists() {
                match mover::send_to_trash(&current) {
                    Ok(()) => {
                        report.restored += 1;
                        undo_records.push(Record {
                            v: 1,
                            ts: now_stamp(),
                            batch: batch.to_string(),
                            state: State::Done,
                            op: Op::Undo,
                            from: current,
                            to: None,
                            size: record.size,
                            category: record.category.clone(),
                            note: None,
                        });
                    }
                    Err(_) => {
                        report.skipped += 1;
                        report.skipped_paths.push(record.from.clone());
                    }
                }
                continue;
            }

            if record.from.exists() {
                report.skipped += 1;
                report.skipped_paths.push(record.from.clone());
                continue;
            }

            match mover::restore(&current, &record.from) {
                Ok(()) => {
                    report.restored += 1;
                    undo_records.push(Record {
                        v: 1,
                        ts: now_stamp(),
                        batch: batch.to_string(),
                        state: State::Done,
                        op: Op::Undo,
                        from: current,
                        to: Some(record.from.clone()),
                        size: record.size,
                        category: record.category.clone(),
                        note: None,
                    });
                }
                Err(_) => {
                    report.skipped += 1;
                    report.skipped_paths.push(record.from.clone());
                }
            }
        }

        self.write(&undo_records)?;
        Ok(report)
    }

    pub fn totals(&self) -> Totals {
        let mut totals = Totals::default();

        for record in &self.records {
            if record.state == State::Done && matches!(record.op, Op::Move | Op::Copy | Op::Trash) {
                totals.files += 1;
                if matches!(record.op, Op::Move | Op::Trash) {
                    totals.bytes += record.size;
                }

                let name = if record.category.is_empty() {
                    "Otros"
                } else {
                    record.category.as_str()
                };

                match totals
                    .categories
                    .iter_mut()
                    .find(|(stored, _)| stored == name)
                {
                    Some((_, count)) => *count += 1,
                    None => totals.categories.push((name.to_string(), 1)),
                }
            }
        }

        totals
            .categories
            .sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        totals
    }

    pub fn all(&self) -> &[Record] {
        &self.records
    }

    pub fn export_csv(&self, path: &Path, lang: crate::i18n::Lang) -> Result<usize, Error> {
        let mut text = crate::i18n::csv_header(lang).to_string();
        text.push('\n');
        let mut rows = 0;

        for record in &self.records {
            let to = record
                .to
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default();
            text.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                csv_field(&record.batch),
                csv_field(&record.ts),
                csv_field(&format!("{:?}", record.state).to_lowercase()),
                csv_field(&format!("{:?}", record.op).to_lowercase()),
                csv_field(&record.from.to_string_lossy()),
                csv_field(&to),
                record.size,
                csv_field(&record.category),
            ));
            rows += 1;
        }

        crate::config::write_text_atomic(path, &text)?;
        Ok(rows)
    }
}

/// Decide que paso con una entrada que quedo pendiente. Nunca mueve ni borra:
/// solo compara lo que hay en disco para dejar un estado honesto.
fn settle(record: &Record) -> (State, Option<String>) {
    match record.op {
        Op::Move | Op::Copy => {
            let Some(to) = record.to.as_ref() else {
                return (State::Failed, None);
            };
            match (record.from.exists(), to.exists()) {
                // El destino existe y el origen ya no: el trabajo termino.
                (false, true) => (State::Done, None),
                // Los dos existen. Puede ser un movimiento interrumpido o una
                // copia que dejo el original; se avisa y no se toca nada.
                (true, true) => (State::Review, Some(review_note(&record.from, to))),
                _ => (State::Failed, None),
            }
        }
        // La papelera y el deshacer no se pueden comprobar sin arriesgar un
        // falso positivo; quedan como interrumpidos.
        Op::Trash | Op::Undo => (State::Failed, None),
    }
}

fn review_note(from: &Path, to: &Path) -> String {
    let (Ok(source), Ok(destination)) = (fs::metadata(from), fs::metadata(to)) else {
        return "unreadable".to_string();
    };
    if source.len() != destination.len() {
        return "different".to_string();
    }
    match (file_hash(from), file_hash(to)) {
        (Some(source), Some(destination)) if source == destination => "identical".to_string(),
        (Some(_), Some(_)) => "different".to_string(),
        _ => "unreadable".to_string(),
    }
}

fn file_hash(path: &Path) -> Option<[u8; 32]> {
    let mut file = File::open(path).ok()?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Some(*hasher.finalize().as_bytes())
}

/// Revisa el diario antes de que la interfaz o el vigilante hagan cualquier
/// cosa. Si no hay diario todavia, no crea uno vacio.
pub fn recover_pending() -> Result<(), Error> {
    let path = crate::config::config_dir()?.join("journal.jsonl");
    if !path.exists() {
        return Ok(());
    }
    let _journal = Journal::open(path)?;
    Ok(())
}

fn csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn now_stamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

pub fn new_batch_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

/// Fecha legible en la zona local resuelta al arranque.
pub fn format_stamp(stamp: &str) -> String {
    let Ok(parsed) = OffsetDateTime::parse(stamp, &Rfc3339) else {
        return stamp.to_string();
    };

    let local = crate::clock::offset()
        .map(|offset| parsed.to_offset(offset))
        .unwrap_or(parsed);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        local.year(),
        u8::from(local.month()),
        local.day(),
        local.hour(),
        local.minute()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending_journal(dir: &Path, from: &Path, to: &Path, size: u64) -> PathBuf {
        let path = dir.join("journal.jsonl");
        let mut journal = Journal::open(path.clone()).expect("open journal");
        journal
            .begin(
                "batch",
                &[NewEntry {
                    from: from.to_path_buf(),
                    to: Some(to.to_path_buf()),
                    size,
                    op: Op::Move,
                    category: "Documentos".to_string(),
                }],
            )
            .expect("begin");
        path
    }

    #[test]
    fn both_files_present_need_review_and_are_not_deleted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let from = dir.path().join("origen.txt");
        let to = dir.path().join("destino.txt");
        fs::write(&from, b"igual").expect("origen");
        fs::write(&to, b"igual").expect("destino");
        let path = pending_journal(dir.path(), &from, &to, 5);

        let journal = Journal::open(path).expect("reopen");
        let record = journal.all().last().expect("record");
        assert_eq!(record.state, State::Review);
        assert_eq!(record.note.as_deref(), Some("identical"));
        assert!(from.exists());
        assert!(to.exists());
    }

    #[test]
    fn different_contents_are_review_with_a_different_note() {
        let dir = tempfile::tempdir().expect("tempdir");
        let from = dir.path().join("origen.txt");
        let to = dir.path().join("destino.txt");
        fs::write(&from, b"uno").expect("origen");
        fs::write(&to, b"dos").expect("destino");
        let path = pending_journal(dir.path(), &from, &to, 3);

        let journal = Journal::open(path).expect("reopen");
        let record = journal.all().last().expect("record");
        assert_eq!(record.state, State::Review);
        assert_eq!(record.note.as_deref(), Some("different"));
    }

    #[test]
    fn missing_source_with_destination_completes_the_move() {
        let dir = tempfile::tempdir().expect("tempdir");
        let from = dir.path().join("origen.txt");
        let to = dir.path().join("destino.txt");
        fs::write(&to, b"contenido").expect("destino");
        let path = pending_journal(dir.path(), &from, &to, 9);

        let journal = Journal::open(path).expect("reopen");
        let record = journal.all().last().expect("record");
        assert_eq!(record.state, State::Done);
    }

    #[test]
    fn pending_without_destination_fails_without_touching_the_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let from = dir.path().join("origen.txt");
        let to = dir.path().join("destino.txt");
        fs::write(&from, b"contenido").expect("origen");
        let path = pending_journal(dir.path(), &from, &to, 9);

        let journal = Journal::open(path).expect("reopen");
        let record = journal.all().last().expect("record");
        assert_eq!(record.state, State::Failed);
        assert!(from.exists());
        assert!(!to.exists());
    }

    #[test]
    fn review_counts_appear_in_the_batch_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let from = dir.path().join("origen.txt");
        let to = dir.path().join("destino.txt");
        fs::write(&from, b"contenido").expect("origen");
        fs::write(&to, b"contenido").expect("destino");
        let path = pending_journal(dir.path(), &from, &to, 9);

        let journal = Journal::open(path).expect("reopen");
        let summary = journal.batch_summaries().into_iter().next().expect("batch");
        assert_eq!(summary.review, 1);
        assert_eq!(summary.files, 0);
    }
}
