//! Recorrido de las carpetas elegidas.
//!
//! No sigue enlaces simbolicos, no entra en carpetas del sistema ni en las
//! carpetas destino de primer nivel de cada raiz, y comprueba que cada archivo
//! siga colgando de su raiz. Las exclusiones y las extensiones temporales son
//! configurables; las carpetas del sistema siguen protegidas siempre.

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use globset::{GlobBuilder, GlobMatcher};

use crate::config::{self, Action, Settings};
use crate::error::Error;

const MAX_DEPTH: usize = 8;

/// Nunca se recorren, aunque alguien las borre de los ajustes.
const SYSTEM_DIRECTORIES: &[&str] = &[
    "$RECYCLE.BIN",
    "System Volume Information",
    "AppData",
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
];

#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    pub root: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub created: Option<SystemTime>,
    pub name: String,
    pub ext: String,
}

impl Candidate {
    pub fn stem(&self) -> String {
        self.path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.name.clone())
    }
}

pub fn scan(settings: &Settings) -> Result<Vec<Candidate>, Error> {
    let mut candidates = Vec::new();

    for root in settings.active_roots() {
        candidates.extend(scan_root(settings, &root)?);
    }

    candidates.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(candidates)
}

/// Recorre una sola raiz con los ajustes efectivos de esa raiz.
pub fn scan_root(settings: &Settings, root: &Path) -> Result<Vec<Candidate>, Error> {
    let root = dunce::canonicalize(root).map_err(|err| Error::io(root, err))?;
    let rules = ScanRules::new(settings, &root);
    let mut candidates = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    walk(&root, &rules, &mut candidates, &mut seen)?;
    candidates.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(candidates)
}

/// Solo los archivos que acaban de aparecer. Evita repetir el arbol entero
/// cuando el vigilante ya sabe exactamente que cambio.
pub fn scan_paths(settings: &Settings, root: &Path, paths: &[PathBuf]) -> Vec<Candidate> {
    let Ok(root) = dunce::canonicalize(root) else {
        return Vec::new();
    };
    let rules = ScanRules::new(settings, &root);
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    for path in paths {
        let Ok(metadata) = fs::symlink_metadata(path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }
        let Some(name) = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            continue;
        };
        if is_hidden(&name, path) || rules.skips_file(&name) || rules.protected_files.contains(path)
        {
            continue;
        }
        let Ok(canonical) = dunce::canonicalize(path) else {
            continue;
        };
        let Ok(relative) = canonical.strip_prefix(&root) else {
            continue;
        };
        let mut safe = true;
        let mut depth = 0usize;
        for component in relative.components() {
            if let Component::Normal(part) = component {
                let part = part.to_string_lossy();
                depth += 1;
                if is_hidden(&part, &root) || rules.skips_dir(&part) {
                    safe = false;
                    break;
                }
                if depth == 1 && rules.is_category_dir(&part) {
                    safe = false;
                    break;
                }
            }
        }
        if !safe || depth > MAX_DEPTH || !inside_root(&canonical, &root) {
            continue;
        }
        if let Some(candidate) = inspect(&canonical, &root, &name, &rules) {
            if seen.insert(candidate.path.clone()) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

struct ScanRules {
    category_dirs: HashSet<String>,
    protected_files: HashSet<PathBuf>,
    exclude_files: Vec<GlobMatcher>,
    exclude_dirs: Vec<GlobMatcher>,
    temp_extensions: HashSet<String>,
}

impl ScanRules {
    fn new(settings: &Settings, root: &Path) -> Self {
        let mut category_dirs = HashSet::new();
        for category in &settings.categories {
            category_dirs.insert(category.folder.clone());
            if let Some(alias) = crate::config::unaccent(&category.folder) {
                category_dirs.insert(alias);
            }
        }

        // Las carpetas destino de las reglas tambien son destinos reales: si se
        // volvieran a recorrer, cada pasada duplicaria los archivos.
        for rule in &settings.rules {
            let folder = match &rule.action {
                Action::MoveTo { value } | Action::CopyTo { value } => Some(value),
                Action::MoveRename { folder, .. } | Action::CopyRename { folder, .. } => {
                    Some(folder)
                }
                Action::Trash | Action::Rename { .. } | Action::Ignore => None,
            };
            let Some(folder) = folder else {
                continue;
            };
            let dest = config::resolve_within(root, folder);
            let Ok(relative) = dest.strip_prefix(root) else {
                continue;
            };
            let Some(Component::Normal(first)) = relative.components().next() else {
                continue;
            };
            let name = first.to_string_lossy().into_owned();
            if let Some(alias) = crate::config::unaccent(&name) {
                category_dirs.insert(alias);
            }
            category_dirs.insert(name);
        }

        // El sistema de archivos de Windows no distingue mayusculas: comparar
        // "Documentos" con "documentos" como texto dejaria entrar a la carpeta
        // destino y volveria a mover sus archivos.
        if cfg!(windows) {
            category_dirs = category_dirs
                .into_iter()
                .map(|name| name.to_lowercase())
                .collect();
        }

        let mut exclude_dirs: Vec<GlobMatcher> = Vec::new();
        for pattern in settings
            .skip_dirs
            .iter()
            .chain(settings.watch.exclude_dirs.iter())
        {
            if let Some(glob) = compile_glob(pattern) {
                exclude_dirs.push(glob);
            }
        }

        let exclude_files = settings
            .watch
            .exclude_files
            .iter()
            .filter_map(|pattern| compile_glob(pattern))
            .collect();

        // En modo portable, la configuracion, el diario y los ejecutables
        // viven dentro de la carpeta elegida. Ninguno puede organizarse a si
        // mismo: mover el proceso en marcha o su diario dejaria la app rota.
        let mut protected_files = HashSet::new();
        if let Ok(dir) = config::config_dir() {
            if let Ok(dir) = dunce::canonicalize(dir) {
                for name in [
                    "config.toml",
                    "config.bak",
                    "journal.jsonl",
                    "error.log",
                    "portable",
                    "watch.stop",
                ] {
                    protected_files.insert(dir.join(name));
                }
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Ok(exe) = dunce::canonicalize(&exe) {
                protected_files.insert(exe.clone());
                if let Some(parent) = exe.parent() {
                    for name in [
                        "Smart Organizer.exe",
                        "smart-organizer.exe",
                        "smart-organizer-cli.exe",
                    ] {
                        protected_files.insert(parent.join(name));
                    }
                }
            }
        }

        Self {
            category_dirs,
            protected_files,
            exclude_files,
            exclude_dirs,
            temp_extensions: settings.temp_extensions.iter().cloned().collect(),
        }
    }

    fn is_category_dir(&self, name: &str) -> bool {
        if cfg!(windows) {
            self.category_dirs.contains(&name.to_lowercase())
        } else {
            self.category_dirs.contains(name)
        }
    }

    fn skips_dir(&self, name: &str) -> bool {
        SYSTEM_DIRECTORIES
            .iter()
            .any(|skip| skip.eq_ignore_ascii_case(name))
            || self.exclude_dirs.iter().any(|glob| glob.is_match(name))
    }

    fn skips_file(&self, name: &str) -> bool {
        self.exclude_files.iter().any(|glob| glob.is_match(name))
    }
}

fn compile_glob(value: &str) -> Option<GlobMatcher> {
    GlobBuilder::new(value)
        .case_insensitive(true)
        .build()
        .ok()
        .map(|glob| glob.compile_matcher())
}

fn walk(
    root: &Path,
    rules: &ScanRules,
    out: &mut Vec<Candidate>,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), Error> {
    let mut stack = vec![(root.to_path_buf(), 0usize)];

    while let Some((dir, depth)) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            // Una subcarpeta sin permiso no cancela el resto del escaneo.
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };

            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();

            if is_hidden(&name, &path) {
                continue;
            }

            if file_type.is_dir() {
                let direct_child = path.parent() == Some(root);
                if depth >= MAX_DEPTH
                    || rules.skips_dir(&name)
                    || (direct_child && rules.is_category_dir(&name))
                {
                    continue;
                }

                stack.push((path, depth + 1));
                continue;
            }

            if !file_type.is_file() || rules.skips_file(&name) {
                continue;
            }

            if rules.protected_files.contains(&path) {
                continue;
            }

            let Some(candidate) = inspect(&path, root, &name, rules) else {
                continue;
            };

            if !inside_root(&candidate.path, root) {
                continue;
            }

            if seen.insert(candidate.path.clone()) {
                out.push(candidate);
            }
        }
    }

    Ok(())
}

fn inspect(path: &Path, root: &Path, name: &str, rules: &ScanRules) -> Option<Candidate> {
    let metadata = fs::metadata(path).ok()?;

    if metadata.len() == 0 {
        return None;
    }

    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy().to_ascii_lowercase()))
        .unwrap_or_default();

    if rules.temp_extensions.contains(&ext) {
        return None;
    }

    Some(Candidate {
        path: path.to_path_buf(),
        root: root.to_path_buf(),
        size: metadata.len(),
        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        created: metadata.created().ok(),
        name: name.to_string(),
        ext,
    })
}

fn inside_root(path: &Path, root: &Path) -> bool {
    let parent = match path.parent() {
        Some(parent) => parent,
        None => return false,
    };

    match dunce::canonicalize(parent) {
        Ok(canonical) => canonical.starts_with(root),
        Err(_) => false,
    }
}

pub fn is_hidden(name: &str, path: &Path) -> bool {
    if name.starts_with('.') {
        return true;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
    }

    false
}

pub fn is_too_recent(candidate: &Candidate, minutes: u64) -> bool {
    if minutes == 0 {
        return false;
    }

    let guard = Duration::from_secs(minutes.saturating_mul(60));
    match SystemTime::now().duration_since(candidate.modified) {
        Ok(age) => age < guard,
        // Un archivo con fecha futura (reloj movido) se trata como reciente.
        Err(_) => true,
    }
}
