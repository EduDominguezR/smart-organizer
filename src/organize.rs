//! Une escaneo, reglas, movimiento y diario en dos pasos: planificar y ejecutar.
//!
//! La interfaz y el CLI llaman a las mismas funciones, asi que un `--dry-run` en
//! consola muestra exactamente lo que la ventana mostraria. Cada raiz se escanea
//! con sus propios ajustes y reglas, si los define.

use std::path::PathBuf;
use std::time::Duration;

use time::OffsetDateTime;

use crate::config::Settings;
use crate::error::Error;
use crate::i18n::{self, Lang};
use crate::journal::{self, Journal, NewEntry, Op, State, UndoReport};
use crate::mover;
use crate::rules::{Engine, Leave, Target};
use crate::scanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanOp {
    Move,
    Copy,
    Trash,
}

#[derive(Debug, Clone)]
pub struct PlannedItem {
    pub from: PathBuf,
    pub to: Option<PathBuf>,
    pub name: String,
    pub category: String,
    pub op: PlanOp,
    pub size: u64,
    pub include: bool,
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub batch: String,
    pub items: Vec<PlannedItem>,
    pub roots: Vec<PathBuf>,
    pub scanned: usize,
    pub left_alone: usize,
}

impl Plan {
    pub fn selected(&self) -> impl Iterator<Item = &PlannedItem> {
        self.items.iter().filter(|item| item.include)
    }

    pub fn selected_count(&self) -> usize {
        self.selected().count()
    }

    pub fn selected_bytes(&self) -> u64 {
        self.selected().map(|item| item.size).sum()
    }

    pub fn trash_count(&self) -> usize {
        self.selected()
            .filter(|item| item.op == PlanOp::Trash)
            .count()
    }

    pub fn move_count(&self) -> usize {
        self.selected()
            .filter(|item| item.op != PlanOp::Trash)
            .count()
    }

    pub fn copy_count(&self) -> usize {
        self.selected()
            .filter(|item| item.op == PlanOp::Copy)
            .count()
    }

    pub fn destinations(&self) -> Vec<String> {
        let mut folders: Vec<String> = Vec::new();
        for item in self.selected().filter(|item| item.op != PlanOp::Trash) {
            if let Some(folder) = item
                .to
                .as_ref()
                .and_then(|to| to.parent())
                .and_then(|parent| parent.file_name())
            {
                let folder = folder.to_string_lossy().into_owned();
                if !folders.contains(&folder) {
                    folders.push(folder);
                }
            }
        }
        folders
    }

    pub fn is_empty(&self) -> bool {
        self.selected_count() == 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub batch: String,
    pub moved: usize,
    pub copied: usize,
    pub trashed: usize,
    pub bytes: u64,
    pub failures: Vec<(PathBuf, String)>,
}

pub fn build(settings: &Settings) -> Result<Plan, Error> {
    build_from(settings, None)
}

/// Planifica solo los archivos que aviso el vigilante. Si no queda ninguno,
/// devuelve un plan vacio sin recorrer el resto de la carpeta.
pub fn build_paths(settings: &Settings, paths: &[PathBuf]) -> Result<Plan, Error> {
    build_from(settings, Some(paths))
}

fn build_from(settings: &Settings, paths: Option<&[PathBuf]>) -> Result<Plan, Error> {
    let roots = settings.active_roots();
    if roots.is_empty() {
        return Err(Error::NoRoots);
    }

    let local = crate::clock::offset().unwrap_or(time::UtcOffset::UTC);
    let now = OffsetDateTime::now_utc().to_offset(local);

    let mut items = Vec::new();
    let mut scanned = 0;
    let mut left_alone = 0;

    for root in &roots {
        let effective = settings.effective_for(root);
        let candidates = match paths {
            Some(paths) => scanner::scan_paths(&effective, root, paths),
            None => scanner::scan_root(&effective, root)?,
        };
        let engine = Engine::new(&effective);
        scanned += candidates.len();

        for candidate in &candidates {
            // El contador de plantilla es estable: si cambiara entre pasadas, una
            // copia con {contador} se repetiria con otro nombre cada vez.
            let counter = 1;
            match engine.decide(candidate, now, counter) {
                Target::Move {
                    dest,
                    name,
                    category,
                } => items.push(PlannedItem {
                    from: candidate.path.clone(),
                    to: Some(dest.join(&name)),
                    name,
                    category,
                    op: PlanOp::Move,
                    size: candidate.size,
                    include: true,
                    root: candidate.root.clone(),
                }),
                Target::Copy {
                    dest,
                    name,
                    category,
                } => items.push(PlannedItem {
                    from: candidate.path.clone(),
                    to: Some(dest.join(&name)),
                    name,
                    category,
                    op: PlanOp::Copy,
                    size: candidate.size,
                    include: true,
                    root: candidate.root.clone(),
                }),
                Target::Trash { category } => items.push(PlannedItem {
                    from: candidate.path.clone(),
                    to: None,
                    name: candidate.name.clone(),
                    category,
                    op: PlanOp::Trash,
                    size: candidate.size,
                    include: true,
                    root: candidate.root.clone(),
                }),
                Target::Leave(Leave::Recent)
                | Target::Leave(Leave::NoCategory)
                | Target::Leave(Leave::AlreadyThere)
                | Target::Leave(Leave::Ignored) => left_alone += 1,
            }
        }
    }

    Ok(Plan {
        batch: journal::new_batch_id(),
        items,
        roots,
        scanned,
        left_alone,
    })
}

pub fn execute(plan: &Plan, stability_ms: u64) -> Result<Summary, Error> {
    execute_with(plan, stability_ms, &crate::config::journal_path()?)
}

/// Igual que `execute`, con el diario en una ruta concreta. Lo usan los tests
/// para no escribir en la configuracion real de la persona.
pub fn execute_with(
    plan: &Plan,
    stability_ms: u64,
    journal_path: &std::path::Path,
) -> Result<Summary, Error> {
    let mut journal = Journal::open(journal_path.to_path_buf())?;
    let mut summary = Summary {
        batch: plan.batch.clone(),
        ..Summary::default()
    };

    let selected: Vec<&PlannedItem> = plan.selected().collect();
    if selected.is_empty() {
        return Ok(summary);
    }

    // Nadie mueve un archivo que todavia se esta escribiendo: dos lecturas de
    // tamano separadas por `stability_ms` tienen que coincidir.
    let unstable = unstable_paths(&selected, stability_ms);

    let entries: Vec<NewEntry> = selected.iter().map(|item| entry_of(item)).collect();
    journal.begin(&plan.batch, &entries)?;

    let mut done: Vec<NewEntry> = Vec::new();
    let mut failed: Vec<NewEntry> = Vec::new();

    for item in &selected {
        let entry = entry_of(item);

        if unstable.iter().any(|path| path == &item.from) {
            failed.push(entry);
            summary
                .failures
                .push((item.from.clone(), String::from("unstable")));
            continue;
        }

        match item.op {
            PlanOp::Move | PlanOp::Trash => {
                if mover::is_locked(&item.from) {
                    failed.push(entry);
                    summary
                        .failures
                        .push((item.from.clone(), String::from("locked")));
                    continue;
                }
            }
            PlanOp::Copy => {}
        }

        match item.op {
            PlanOp::Move => {
                let Some(dest) = item.to.as_ref() else {
                    failed.push(entry);
                    continue;
                };
                let Some(dest_dir) = dest.parent() else {
                    failed.push(entry);
                    continue;
                };

                match mover::perform_move(&item.from, dest_dir, &item.name) {
                    Ok(final_path) => {
                        summary.moved += 1;
                        summary.bytes += item.size;
                        done.push(NewEntry {
                            to: Some(final_path),
                            ..entry
                        });
                    }
                    Err(err) => {
                        summary.failures.push((
                            item.from.clone(),
                            err.describe(crate::i18n::Lang::default()),
                        ));
                        failed.push(entry);
                    }
                }
            }
            PlanOp::Copy => {
                let Some(dest) = item.to.as_ref() else {
                    failed.push(entry);
                    continue;
                };
                let Some(dest_dir) = dest.parent() else {
                    failed.push(entry);
                    continue;
                };

                match mover::perform_copy(&item.from, dest_dir, &item.name) {
                    Ok(final_path) => {
                        summary.copied += 1;
                        done.push(NewEntry {
                            to: Some(final_path),
                            ..entry
                        });
                    }
                    Err(err) => {
                        summary.failures.push((
                            item.from.clone(),
                            err.describe(crate::i18n::Lang::default()),
                        ));
                        failed.push(entry);
                    }
                }
            }
            PlanOp::Trash => match mover::send_to_trash(&item.from) {
                Ok(()) => {
                    summary.trashed += 1;
                    summary.bytes += item.size;
                    done.push(entry);
                }
                Err(err) => {
                    summary.failures.push((
                        item.from.clone(),
                        err.describe(crate::i18n::Lang::default()),
                    ));
                    failed.push(entry);
                }
            },
        }
    }

    journal.mark(&plan.batch, &done, State::Done)?;
    journal.mark(&plan.batch, &failed, State::Failed)?;

    Ok(summary)
}

fn entry_of(item: &PlannedItem) -> NewEntry {
    NewEntry {
        from: item.from.clone(),
        to: item.to.clone(),
        size: item.size,
        op: match item.op {
            PlanOp::Move => Op::Move,
            PlanOp::Copy => Op::Copy,
            PlanOp::Trash => Op::Trash,
        },
        category: item.category.clone(),
    }
}

fn unstable_paths(items: &[&PlannedItem], stability_ms: u64) -> Vec<PathBuf> {
    if stability_ms == 0 {
        return Vec::new();
    }

    let sizes = |items: &[&PlannedItem]| -> Vec<(PathBuf, Option<u64>)> {
        items
            .iter()
            .map(|item| {
                let size = std::fs::metadata(&item.from).map(|meta| meta.len()).ok();
                (item.from.clone(), size)
            })
            .collect()
    };

    let before = sizes(items);
    std::thread::sleep(Duration::from_millis(stability_ms));
    let after = sizes(items);

    let mut changed = Vec::new();
    for ((path, first), (_, second)) in before.iter().zip(after.iter()) {
        if first != second || second.is_none() {
            changed.push(path.clone());
        }
    }

    changed
}

pub fn undo_last() -> Result<UndoReport, Error> {
    let mut journal = Journal::open(crate::config::journal_path()?)?;
    match journal.last_batch() {
        Some(batch) => journal.undo(&batch),
        None => Ok(UndoReport::default()),
    }
}

pub fn undo_batch(batch: &str) -> Result<UndoReport, Error> {
    undo_batch_with(batch, &crate::config::journal_path()?)
}

pub fn undo_batch_with(batch: &str, journal_path: &std::path::Path) -> Result<UndoReport, Error> {
    let mut journal = Journal::open(journal_path.to_path_buf())?;
    journal.undo(batch)
}

pub fn history() -> Result<Vec<journal::BatchSummary>, Error> {
    let journal = Journal::open(crate::config::journal_path()?)?;
    Ok(journal.batch_summaries())
}

pub fn totals() -> Result<journal::Totals, Error> {
    let journal = Journal::open(crate::config::journal_path()?)?;
    Ok(journal.totals())
}

pub fn export_csv(path: &std::path::Path, lang: Lang) -> Result<usize, Error> {
    let journal = Journal::open(crate::config::journal_path()?)?;
    journal.export_csv(path, lang)
}

/// Texto de una linea para la consola: origen -> destino.
pub fn describe_item(item: &PlannedItem, lang: Lang) -> String {
    match item.op {
        PlanOp::Move => {
            let dest = item
                .to
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default();
            format!("{}  ->  {}", item.from.display(), dest)
        }
        PlanOp::Copy => {
            let dest = item
                .to
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default();
            format!(
                "{}  ->  {}  [{}]",
                item.from.display(),
                dest,
                i18n::t(lang, i18n::Key::ActionCopy)
            )
        }
        PlanOp::Trash => format!(
            "{}  ->  {}",
            item.from.display(),
            i18n::t(lang, i18n::Key::Trash)
        ),
    }
}

/// Conteo por categoria, para la grafica de la vista previa.
pub fn categories(plan: &Plan) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for item in plan.selected() {
        match out.iter_mut().find(|(name, _)| *name == item.category) {
            Some((_, count)) => *count += 1,
            None => out.push((item.category.clone(), 1)),
        }
    }
    out.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    out
}
