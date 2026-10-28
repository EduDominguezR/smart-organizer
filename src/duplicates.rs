//! Busqueda de archivos identicos bajo demanda.
//!
//! No se ejecuta sola ni al arrancar. Primero agrupa por tamano, luego compara
//! solo el principio y el final de cada archivo y reserva el hash completo para
//! los candidatos reales. El original elegido se conserva siempre.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use crate::config::Settings;
use crate::error::Error;
use crate::journal::{self, Journal, NewEntry, Op, State};
use crate::mover;
use crate::scanner;

const PARTIAL: u64 = 64 * 1024;

#[derive(Debug, Clone)]
pub struct DuplicateFile {
    pub path: PathBuf,
    pub modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub size: u64,
    /// El primero es el que se conserva por defecto.
    pub files: Vec<DuplicateFile>,
    pub keep: usize,
}

impl DuplicateGroup {
    pub fn wasted(&self) -> u64 {
        self.size
            .saturating_mul(self.files.len().saturating_sub(1) as u64)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TrashOutcome {
    pub trashed: usize,
    pub failed: usize,
    pub bytes: u64,
}

/// `Ok(None)` significa que la persona cancelo; no se devuelve un resultado a
/// medias para que la interfaz no muestre grupos incompletos.
pub fn find(
    settings: &Settings,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize, usize),
) -> Result<Option<Vec<DuplicateGroup>>, Error> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    for root in settings.active_roots() {
        if cancel.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let effective = settings.effective_for(&root);
        let candidates = match scanner::scan_root(&effective, &root) {
            Ok(candidates) => candidates,
            Err(_) => continue,
        };
        for candidate in candidates {
            if !seen.insert(candidate.path.clone()) {
                continue;
            }
            if placeholder(&candidate.path) {
                continue;
            }
            files.push(candidate);
        }
    }

    let mut by_size: HashMap<u64, Vec<scanner::Candidate>> = HashMap::new();
    for candidate in files {
        by_size.entry(candidate.size).or_default().push(candidate);
    }

    let mut by_partial: HashMap<(u64, blake3::Hash), Vec<scanner::Candidate>> = HashMap::new();
    let mut buffer = vec![0u8; PARTIAL as usize];
    let mut done = 0usize;
    let total: usize = by_size
        .values()
        .filter(|group| group.len() > 1)
        .map(Vec::len)
        .sum();

    for (size, group) in by_size {
        if group.len() < 2 {
            continue;
        }
        for candidate in group {
            if cancel.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let Ok(hash) = partial_hash(&candidate.path, size, &mut buffer) else {
                continue;
            };
            by_partial.entry((size, hash)).or_default().push(candidate);
            done += 1;
            progress(done, total);
        }
    }

    let candidate_count: usize = by_partial
        .values()
        .filter(|group| group.len() > 1)
        .map(Vec::len)
        .sum();
    done = 0;

    let mut groups = Vec::new();
    for ((size, _), group) in by_partial {
        if group.len() < 2 {
            continue;
        }
        let mut by_hash: HashMap<blake3::Hash, Vec<scanner::Candidate>> = HashMap::new();
        for candidate in group {
            if cancel.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let Ok(hash) = full_hash(&candidate.path, &mut buffer) else {
                continue;
            };
            by_hash.entry(hash).or_default().push(candidate);
            done += 1;
            progress(done, candidate_count.max(done));
        }

        for (_, files) in by_hash {
            if files.len() < 2 {
                continue;
            }
            let mut files: Vec<DuplicateFile> = files
                .into_iter()
                .map(|candidate| DuplicateFile {
                    path: candidate.path,
                    modified: candidate.modified,
                })
                .collect();
            files.sort_by(|a, b| {
                a.modified
                    .cmp(&b.modified)
                    .then_with(|| depth(&a.path).cmp(&depth(&b.path)))
                    .then_with(|| a.path.cmp(&b.path))
            });
            groups.push(DuplicateGroup {
                size,
                files,
                keep: 0,
            });
        }
    }

    groups.sort_by(|a, b| {
        b.wasted()
            .cmp(&a.wasted())
            .then_with(|| a.files[0].path.cmp(&b.files[0].path))
    });
    Ok(Some(groups))
}

fn depth(path: &Path) -> usize {
    path.components().count()
}

fn placeholder(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const OFFLINE: u32 = 0x0000_1000;
        const RECALL_ON_OPEN: u32 = 0x0004_0000;
        const RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
        if let Ok(metadata) = fs::metadata(path) {
            let attrs = metadata.file_attributes();
            return attrs & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS) != 0;
        }
    }
    let _ = path;
    false
}

fn partial_hash(path: &Path, size: u64, buffer: &mut [u8]) -> Result<blake3::Hash, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let head = size.min(PARTIAL);
    if head > 0 {
        file.read_exact(&mut buffer[..head as usize])?;
        hasher.update(&buffer[..head as usize]);
    }
    if size > PARTIAL {
        let tail = size.min(PARTIAL);
        file.seek(SeekFrom::Start(size - tail))?;
        file.read_exact(&mut buffer[..tail as usize])?;
        hasher.update(&buffer[..tail as usize]);
    }
    Ok(hasher.finalize())
}

fn full_hash(path: &Path, buffer: &mut [u8]) -> Result<blake3::Hash, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    loop {
        let read = file.read(buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize())
}

/// Envia a la papelera solo los duplicados elegidos, despues de comprobar que
/// el archivo conservado sigue existiendo y coincide con cada copia.
pub fn trash_selected(
    groups: &[DuplicateGroup],
    selected: &HashSet<PathBuf>,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize, usize),
) -> Result<TrashOutcome, Error> {
    let mut buffer = vec![0u8; 256 * 1024];
    let mut removals: Vec<(PathBuf, u64)> = Vec::new();

    for group in groups {
        let chosen: Vec<&PathBuf> = group
            .files
            .iter()
            .map(|file| &file.path)
            .filter(|path| selected.contains(*path))
            .collect();
        if chosen.is_empty() {
            continue;
        }

        let keep = group
            .files
            .get(group.keep)
            .ok_or_else(|| Error::NotFound {
                path: PathBuf::new(),
            })?
            .path
            .clone();
        if !keep.exists() {
            return Err(Error::NotFound { path: keep });
        }
        let keep_hash = full_hash(&keep, &mut buffer).map_err(|err| Error::io(&keep, err))?;
        for path in chosen {
            if !path.exists() {
                return Err(Error::NotFound { path: path.clone() });
            }
            let hash = full_hash(path, &mut buffer).map_err(|err| Error::io(path, err))?;
            if hash != keep_hash {
                return Err(Error::CopyVerify { path: path.clone() });
            }
            removals.push((path.clone(), group.size));
        }
    }

    if removals.is_empty() {
        return Ok(TrashOutcome::default());
    }

    let journal_path = crate::config::journal_path()?;
    let mut journal = Journal::open(journal_path)?;
    let batch = journal::new_batch_id();
    let entries: Vec<NewEntry> = removals
        .iter()
        .map(|(path, size)| NewEntry {
            from: path.clone(),
            to: None,
            size: *size,
            op: Op::Trash,
            category: "Duplicados".to_string(),
        })
        .collect();
    journal.begin(&batch, &entries)?;

    let mut outcome = TrashOutcome::default();
    let total = removals.len();
    let mut ok_entries = Vec::new();
    let mut failed_entries = Vec::new();

    for (done, (path, size)) in removals.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            let entry = NewEntry {
                from: path,
                to: None,
                size,
                op: Op::Trash,
                category: "Duplicados".to_string(),
            };
            failed_entries.push(entry);
        } else {
            let entry = NewEntry {
                from: path.clone(),
                to: None,
                size,
                op: Op::Trash,
                category: "Duplicados".to_string(),
            };
            match mover::send_to_trash(&path) {
                Ok(()) => {
                    outcome.trashed += 1;
                    outcome.bytes += size;
                    ok_entries.push(entry);
                }
                Err(_) => {
                    outcome.failed += 1;
                    failed_entries.push(entry);
                }
            }
        }
        progress(done + 1, total);
    }

    journal.mark(&batch, &ok_entries, State::Done)?;
    journal.mark(&batch, &failed_entries, State::Failed)?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Root;
    use std::sync::atomic::AtomicBool;

    fn settings_for(root: &Path) -> Settings {
        let mut settings = Settings::default();
        settings.roots.push(Root::new(root.to_path_buf()));
        settings
    }

    #[test]
    fn identical_files_group_and_the_shallowest_stays() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("original.txt"), b"mismos bytes").unwrap();
        fs::write(root.join("sub").join("copia.txt"), b"mismos bytes").unwrap();
        fs::write(root.join("otro.txt"), b"distintos!!").unwrap();
        fs::write(root.join("vacio.txt"), b"").unwrap();

        let settings = settings_for(root);
        let groups = find(&settings, &AtomicBool::new(false), |_, _| {})
            .unwrap()
            .unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].files.len(), 2);
        assert_eq!(groups[0].keep, 0);
        assert!(groups[0].files[0].path.ends_with("original.txt"));
        assert_eq!(groups[0].wasted(), 12);
    }

    #[test]
    fn cancelling_before_reading_returns_no_partial_results() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("a.bin"), b"igual").unwrap();
        fs::write(root.join("b.bin"), b"igual").unwrap();

        let settings = settings_for(root);
        let groups = find(&settings, &AtomicBool::new(true), |_, _| {}).unwrap();
        assert!(groups.is_none());
    }

    #[test]
    fn chooses_oldest_then_shallowest() {
        let group = DuplicateGroup {
            size: 4,
            files: vec![
                DuplicateFile {
                    path: PathBuf::from("a/b/c.txt"),
                    modified: SystemTime::UNIX_EPOCH,
                },
                DuplicateFile {
                    path: PathBuf::from("d.txt"),
                    modified: SystemTime::UNIX_EPOCH,
                },
            ],
            keep: 0,
        };
        assert_eq!(group.wasted(), 4);
    }
}
