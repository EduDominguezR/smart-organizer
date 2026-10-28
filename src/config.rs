//! Ajustes del usuario: carpetas, categorias, reglas y vigilancia.
//!
//! El TOML es texto legible, pero la via normal es la interfaz. Los valores por
//! defecto son seguros: sin carpetas elegidas y sin reglas que borren nada.
//!
//! La version 2 anade fecha de origen de las subcarpetas, categoria de reserva
//! explicita, condiciones multiples por regla, acciones de copia e ignorar,
//! ajustes por carpeta y perfiles. Los archivos de la version 1 se leen tal cual
//! y se migran en memoria al cargar.

use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Error;

pub const CONFIG_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    #[default]
    Modified,
    Created,
    Today,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSize {
    Small,
    #[default]
    Normal,
    Large,
}

impl TextSize {
    pub fn scale(self) -> f32 {
        match self {
            Self::Small => 0.9,
            Self::Normal => 1.0,
            Self::Large => 1.15,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Subfolders {
    pub by_year_month: bool,
    pub by_type: bool,
    pub by_capture: bool,
    pub date_source: DateSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub folder: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// La categoria que recoge lo que no encaja en ninguna otra.
    #[serde(default)]
    pub fallback: bool,
    /// Color opcional de la grafica de Inicio, en formato #rrggbb.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Category {
    pub fn new(name: &str, folder: &str) -> Self {
        Self {
            name: name.to_string(),
            folder: folder.to_string(),
            extensions: Vec::new(),
            enabled: true,
            fallback: false,
            color: None,
        }
    }
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RootOverrides {
    pub subfolders: Option<Subfolders>,
    pub exclude_files: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub categories: Option<Vec<Category>>,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub path: PathBuf,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub overrides: RootOverrides,
}

impl Root {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            enabled: true,
            overrides: RootOverrides::default(),
        }
    }
}

/// Formato de la version 1. Solo se usa para migrar ajustes viejos.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Matcher {
    Extension { value: String },
    Glob { value: String },
    MinSize { value: u64 },
    OlderThan { value: u32 },
    Regex { value: String },
}

impl Matcher {
    fn to_condition(&self) -> Condition {
        match self {
            Self::Extension { value } => Condition::Extension {
                value: normalize_extension(value),
            },
            Self::Glob { value } => Condition::Glob {
                value: value.clone(),
            },
            Self::MinSize { value } => Condition::MinSize { value: *value },
            Self::OlderThan { value } => Condition::OlderThan { value: *value },
            Self::Regex { value } => Condition::Regex {
                value: value.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Condition {
    Extension { value: String },
    Glob { value: String },
    NameContains { value: String },
    NameStartsWith { value: String },
    NameEndsWith { value: String },
    MinSize { value: u64 },
    MaxSize { value: u64 },
    OlderThan { value: u32 },
    NewerThan { value: u32 },
    CreatedOlderThan { value: u32 },
    CreatedNewerThan { value: u32 },
    RealType { value: String },
    SourceFolder { value: String },
    Regex { value: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    MoveTo { value: String },
    Trash,
    Rename { value: String },
    CopyTo { value: String },
    MoveRename { folder: String, template: String },
    CopyRename { folder: String, template: String },
    Ignore,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuleSubfolder {
    pub enabled: bool,
    pub by_year_month: bool,
    pub by_type: bool,
    pub date_source: DateSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: u64,
    pub name: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    /// true = todas las condiciones (Y); false = alguna (O).
    #[serde(default = "yes")]
    pub match_all: bool,
    pub action: Action,
    #[serde(default)]
    pub subfolder: RuleSubfolder,
    /// Matcher de la version 1. Se migra a `conditions` al cargar y no se
    /// vuelve a escribir.
    #[serde(rename = "matcher", default, skip_serializing_if = "Option::is_none")]
    pub legacy_matcher: Option<Matcher>,
}

impl Rule {
    pub fn new(id: u64, name: &str, action: Action) -> Self {
        Self {
            id,
            name: name.to_string(),
            enabled: true,
            conditions: Vec::new(),
            match_all: true,
            action,
            subfolder: RuleSubfolder::default(),
            legacy_matcher: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watch {
    /// Espera sin cambios antes de mirar los archivos, en milisegundos.
    pub debounce_ms: u64,
    /// Dos lecturas de tamano separadas por este tiempo tienen que coincidir.
    pub stability_ms: u64,
    /// Los archivos mas nuevos que esto no se tocan.
    pub recent_guard_minutes: u64,
    /// Organizacion periodica ademas de la vigilancia. 0 la apaga.
    pub schedule_minutes: u32,
    /// Nombres o globs de carpetas que no se recorren.
    #[serde(default)]
    pub exclude_dirs: Vec<String>,
    /// Globs de archivos que no se tocan.
    #[serde(default)]
    pub exclude_files: Vec<String>,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            debounce_ms: 1200,
            stability_ms: 500,
            recent_guard_minutes: 0,
            schedule_minutes: 0,
            exclude_dirs: Vec::new(),
            exclude_files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub theme: Theme,
    pub accent: String,
    pub language: String,
    pub text_size: TextSize,
    pub density: Density,
    pub advanced: bool,
    pub silent: bool,
    /// Arranca la vigilancia en un proceso aparte al abrir la ventana.
    pub background: bool,
    /// Registra el vigilante oculto en el arranque de Windows.
    #[serde(default)]
    pub start_with_windows: bool,
    /// La persona ya vio o salto la guia inicial.
    #[serde(default)]
    pub onboarded: bool,
    /// Tamano de la ventana en puntos, recordado entre sesiones.
    #[serde(default = "default_window_size")]
    pub window_size: (f32, f32),
    /// Posicion de la ventana en puntos. Negativa significa centrar.
    #[serde(default = "default_window_pos")]
    pub window_pos: (f32, f32),
    pub roots: Vec<Root>,
    pub categories: Vec<Category>,
    pub rules: Vec<Rule>,
    pub subfolders: Subfolders,
    pub watch: Watch,
    /// Extensiones que el escaner considera descargas a medias.
    #[serde(default = "default_temp_extensions")]
    pub temp_extensions: Vec<String>,
    /// Carpetas que se omiten ademas de las protegidas del sistema.
    #[serde(default = "default_skip_dirs")]
    pub skip_dirs: Vec<String>,
}

fn default_window_size() -> (f32, f32) {
    (1180.0, 720.0)
}

fn default_window_pos() -> (f32, f32) {
    (-1.0, -1.0)
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            theme: Theme::Dark,
            accent: "#2563eb".to_string(),
            language: "es".to_string(),
            text_size: TextSize::Normal,
            density: Density::Comfortable,
            advanced: false,
            silent: false,
            background: false,
            start_with_windows: false,
            onboarded: false,
            window_size: default_window_size(),
            window_pos: default_window_pos(),
            roots: Vec::new(),
            categories: default_categories(),
            rules: Vec::new(),
            subfolders: Subfolders::default(),
            watch: Watch::default(),
            temp_extensions: default_temp_extensions(),
            skip_dirs: default_skip_dirs(),
        }
    }
}

pub fn default_temp_extensions() -> Vec<String> {
    [
        ".crdownload",
        ".part",
        ".tmp",
        ".download",
        ".opdownload",
        ".partial",
        ".!ut",
        ".aria2",
    ]
    .iter()
    .map(|ext| (*ext).to_string())
    .collect()
}

pub fn default_skip_dirs() -> Vec<String> {
    [
        "node_modules",
        ".git",
        "$RECYCLE.BIN",
        "System Volume Information",
        "AppData",
        "Windows",
        "Program Files",
        "Program Files (x86)",
        "ProgramData",
    ]
    .iter()
    .map(|name| (*name).to_string())
    .collect()
}

pub fn default_categories() -> Vec<Category> {
    let table: &[(&str, &str, &[&str])] = &[
        (
            "Documentos",
            "Documentos",
            &[
                ".pdf", ".docx", ".doc", ".txt", ".xlsx", ".xls", ".pptx", ".ppt", ".odt", ".rtf",
                ".csv", ".epub",
            ],
        ),
        (
            "Programas",
            "Programas",
            &[".exe", ".msi", ".dmg", ".pkg", ".deb", ".rpm", ".appimage"],
        ),
        (
            "Comprimidos",
            "Comprimidos",
            &[".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".iso"],
        ),
        (
            "Imagenes",
            "Imágenes",
            &[
                ".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp", ".svg", ".ico", ".tiff", ".heic",
                ".avif",
            ],
        ),
        (
            "Videos",
            "Videos",
            &[
                ".mp4", ".mkv", ".mov", ".avi", ".wmv", ".flv", ".webm", ".m4v", ".3gp",
            ],
        ),
        (
            "Musica",
            "Música",
            &[
                ".mp3", ".wav", ".flac", ".aac", ".ogg", ".wma", ".m4a", ".opus",
            ],
        ),
        (
            "Proyectos",
            "Proyectos",
            &[
                ".py", ".js", ".ts", ".html", ".css", ".java", ".cpp", ".c", ".h", ".json", ".xml",
                ".sql", ".php", ".rb", ".go", ".rs", ".sh", ".bat", ".ps1",
            ],
        ),
        ("Otros", "Otros", &[]),
    ];

    table
        .iter()
        .map(|(name, folder, extensions)| Category {
            name: (*name).to_string(),
            folder: (*folder).to_string(),
            extensions: extensions.iter().map(|ext| (*ext).to_string()).collect(),
            enabled: true,
            fallback: *name == "Otros",
            color: None,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    General,
    Student,
    Developer,
    Photographer,
}

impl Profile {
    pub const ALL: [Profile; 4] = [
        Profile::General,
        Profile::Student,
        Profile::Developer,
        Profile::Photographer,
    ];

    pub fn build(self) -> (Vec<Category>, Vec<Rule>) {
        match self {
            Self::General => (default_categories(), Vec::new()),
            Self::Student => (
                categories(&[
                    (
                        "Apuntes",
                        "Apuntes",
                        &[".pdf", ".docx", ".doc", ".txt", ".odt", ".rtf"],
                    ),
                    (
                        "Presentaciones",
                        "Presentaciones",
                        &[".pptx", ".ppt", ".key", ".odp"],
                    ),
                    (
                        "Hojas de calculo",
                        "Hojas de cálculo",
                        &[".xlsx", ".xls", ".csv", ".ods"],
                    ),
                    ("Libros", "Libros", &[".epub", ".mobi", ".azw3"]),
                    (
                        "Imagenes",
                        "Imágenes",
                        &[".jpg", ".jpeg", ".png", ".gif", ".webp", ".tiff"],
                    ),
                    (
                        "Videos",
                        "Videos",
                        &[".mp4", ".mkv", ".mov", ".avi", ".webm"],
                    ),
                    (
                        "Musica",
                        "Música",
                        &[".mp3", ".wav", ".flac", ".m4a", ".ogg"],
                    ),
                    (
                        "Comprimidos",
                        "Comprimidos",
                        &[".zip", ".rar", ".7z", ".tar", ".gz"],
                    ),
                    ("Programas", "Programas", &[".exe", ".msi"]),
                    ("Otros", "Otros", &[]),
                ]),
                vec![
                    Rule {
                        conditions: vec![Condition::NameContains {
                            value: "factura".to_string(),
                        }],
                        ..Rule::new(
                            1,
                            "Facturas",
                            Action::MoveTo {
                                value: "Apuntes/Facturas".to_string(),
                            },
                        )
                    },
                    Rule {
                        conditions: vec![Condition::NameStartsWith {
                            value: "captura".to_string(),
                        }],
                        ..Rule::new(
                            2,
                            "Capturas",
                            Action::MoveTo {
                                value: "Imágenes/Capturas".to_string(),
                            },
                        )
                    },
                ],
            ),
            Self::Developer => (
                categories(&[
                    (
                        "Proyectos",
                        "Proyectos",
                        &[
                            ".rs", ".py", ".js", ".ts", ".html", ".css", ".java", ".cpp", ".c",
                            ".h", ".go", ".rb", ".php", ".sh", ".ps1", ".toml", ".xml",
                        ],
                    ),
                    (
                        "Datos",
                        "Datos",
                        &[".csv", ".json", ".db", ".sqlite", ".parquet", ".sql"],
                    ),
                    (
                        "Documentos",
                        "Documentos",
                        &[".pdf", ".docx", ".txt", ".md", ".odt", ".rtf"],
                    ),
                    (
                        "Imagenes",
                        "Imágenes",
                        &[".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg"],
                    ),
                    ("Videos", "Videos", &[".mp4", ".mkv", ".mov", ".webm"]),
                    ("Musica", "Música", &[".mp3", ".wav", ".flac", ".ogg"]),
                    (
                        "Comprimidos",
                        "Comprimidos",
                        &[".zip", ".rar", ".7z", ".tar", ".gz"],
                    ),
                    ("Programas", "Programas", &[".exe", ".msi", ".deb", ".rpm"]),
                    ("Otros", "Otros", &[]),
                ]),
                vec![Rule {
                    conditions: vec![Condition::NameStartsWith {
                        value: "screenshot".to_string(),
                    }],
                    ..Rule::new(
                        1,
                        "Capturas",
                        Action::MoveTo {
                            value: "Imágenes/Capturas".to_string(),
                        },
                    )
                }],
            ),
            Self::Photographer => (
                categories(&[
                    (
                        "Fotos",
                        "Fotos",
                        &[
                            ".jpg", ".jpeg", ".png", ".heic", ".tiff", ".raw", ".cr2", ".nef",
                            ".arw", ".dng", ".webp", ".avif",
                        ],
                    ),
                    (
                        "Editables",
                        "Proyectos",
                        &[".psd", ".ai", ".xcf", ".blend", ".indd", ".sketch"],
                    ),
                    (
                        "Videos",
                        "Videos",
                        &[".mp4", ".mov", ".mkv", ".avi", ".webm"],
                    ),
                    (
                        "Documentos",
                        "Documentos",
                        &[".pdf", ".docx", ".txt", ".rtf"],
                    ),
                    ("Comprimidos", "Comprimidos", &[".zip", ".rar", ".7z"]),
                    ("Otros", "Otros", &[]),
                ]),
                vec![Rule {
                    conditions: vec![
                        Condition::Extension {
                            value: ".cr2".to_string(),
                        },
                        Condition::Extension {
                            value: ".nef".to_string(),
                        },
                        Condition::Extension {
                            value: ".arw".to_string(),
                        },
                        Condition::Extension {
                            value: ".dng".to_string(),
                        },
                    ],
                    match_all: false,
                    ..Rule::new(
                        1,
                        "RAW",
                        Action::MoveTo {
                            value: "Fotos/RAW".to_string(),
                        },
                    )
                }],
            ),
        }
    }
}

fn categories(table: &[(&str, &str, &[&str])]) -> Vec<Category> {
    table
        .iter()
        .map(|(name, folder, extensions)| Category {
            name: (*name).to_string(),
            folder: (*folder).to_string(),
            extensions: extensions.iter().map(|ext| (*ext).to_string()).collect(),
            enabled: true,
            fallback: *name == "Otros",
            color: None,
        })
        .collect()
}

impl Settings {
    /// Añade solo lo que falta de un perfil. Devuelve (grupos, reglas, omitidos)
    /// para poder explicar exactamente que se hizo antes de guardar.
    pub fn apply_profile(&mut self, profile: Profile) -> (usize, usize, usize) {
        let (categories, rules) = profile.build();
        let mut added_groups = 0usize;
        let mut added_rules = 0usize;
        let mut skipped = 0usize;
        let mut used_extensions: Vec<String> = self
            .categories
            .iter()
            .flat_map(|category| category.extensions.iter().cloned())
            .map(|extension| extension.to_ascii_lowercase())
            .collect();

        for mut category in categories {
            let name = category.name.to_lowercase();
            let folder = category.folder.to_lowercase();
            if self.categories.iter().any(|existing| {
                existing.name.to_lowercase() == name || existing.folder.to_lowercase() == folder
            }) {
                skipped += 1;
                continue;
            }

            category.extensions.retain(|extension| {
                let extension = extension.to_ascii_lowercase();
                if used_extensions.contains(&extension) {
                    false
                } else {
                    used_extensions.push(extension);
                    true
                }
            });
            if !category.extensions.is_empty() || !category.fallback {
                self.categories.push(category);
                added_groups += 1;
            } else {
                skipped += 1;
            }
        }

        let mut next_id = self
            .rules
            .iter()
            .map(|rule| rule.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        for mut rule in rules {
            if self.rules.iter().any(|existing| {
                existing.name.eq_ignore_ascii_case(&rule.name)
                    || (existing.match_all == rule.match_all
                        && existing.conditions == rule.conditions
                        && existing.action == rule.action)
            }) {
                skipped += 1;
                continue;
            }
            rule.id = next_id;
            next_id = next_id.saturating_add(1);
            self.rules.push(rule);
            added_rules += 1;
        }

        (added_groups, added_rules, skipped)
    }

    pub fn normalize(&mut self) {
        self.version = CONFIG_VERSION;

        if self.language != "en" {
            self.language = "es".to_string();
        }

        if let Some(color) = normalize_color(&self.accent) {
            self.accent = color;
        } else {
            self.accent = "#2563eb".to_string();
        }

        self.window_size.0 = self.window_size.0.clamp(900.0, 10_000.0);
        self.window_size.1 = self.window_size.1.clamp(560.0, 10_000.0);
        if !self.window_pos.0.is_finite() || !self.window_pos.1.is_finite() {
            self.window_pos = default_window_pos();
        }

        normalize_categories(&mut self.categories);
        normalize_rules(&mut self.rules);

        let mut seen_paths: Vec<PathBuf> = Vec::new();
        self.roots.retain_mut(|root| {
            let Some(path) = absolute_path(&root.path) else {
                return false;
            };
            let path = if path.exists() {
                dunce::canonicalize(&path).unwrap_or(path)
            } else {
                path
            };
            if is_protected_root(&path) {
                return false;
            }
            if seen_paths.iter().any(|seen| same_path(seen, &path)) {
                return false;
            }
            seen_paths.push(path.clone());
            root.path = path;
            true
        });

        for root in &mut self.roots {
            if let Some(categories) = &mut root.overrides.categories {
                normalize_categories(categories);
            }
            if let Some(rules) = &mut root.overrides.rules {
                normalize_rules(rules);
            }
            root.overrides.exclude_files = clean_patterns(&root.overrides.exclude_files);
            root.overrides.exclude_dirs = clean_patterns(&root.overrides.exclude_dirs);
        }

        self.temp_extensions = self
            .temp_extensions
            .iter()
            .map(|ext| normalize_extension(ext))
            .filter(|ext| !ext.is_empty())
            .collect();
        self.temp_extensions.sort();
        self.temp_extensions.dedup();

        self.skip_dirs = clean_patterns(&self.skip_dirs);
        self.watch.exclude_files = clean_patterns(&self.watch.exclude_files);
        self.watch.exclude_dirs = clean_patterns(&self.watch.exclude_dirs);

        self.watch.debounce_ms = self.watch.debounce_ms.clamp(200, 30_000);
        self.watch.stability_ms = self.watch.stability_ms.clamp(100, 10_000);
        self.watch.schedule_minutes = self.watch.schedule_minutes.min(1440);
        self.watch.recent_guard_minutes = self.watch.recent_guard_minutes.min(10_080);
        self.roots.retain(|root| !root.path.as_os_str().is_empty());
        self.rules.retain(|rule| !rule.name.trim().is_empty());
    }

    pub fn active_roots(&self) -> Vec<PathBuf> {
        self.roots
            .iter()
            .filter(|root| root.enabled)
            .map(|root| root.path.clone())
            .collect()
    }

    pub fn enabled_rules(&self) -> usize {
        let categories = self.categories.iter().filter(|c| c.enabled).count();
        categories + self.rules.iter().filter(|r| r.enabled).count()
    }

    pub fn next_rule_id(&self) -> u64 {
        let global = self.rules.iter().map(|rule| rule.id).max().unwrap_or(0);
        let roots = self
            .roots
            .iter()
            .filter_map(|root| root.overrides.rules.as_ref())
            .flat_map(|rules| rules.iter().map(|rule| rule.id))
            .max()
            .unwrap_or(0);
        global.max(roots) + 1
    }

    pub fn category_folder(&self, root: &Path, category: &Category) -> PathBuf {
        // Si en el disco ya existe la carpeta sin tilde, se usa esa para no partir
        // la coleccion en dos. Se prefiere siempre el nombre con tilde.
        let preferred = root.join(&category.folder);
        if preferred.exists() {
            return preferred;
        }

        if let Some(alias) = unaccent(&category.folder) {
            let other = root.join(alias);
            if other.exists() {
                return other;
            }
        }

        preferred
    }

    pub fn fallback_category(&self) -> Option<&Category> {
        self.categories
            .iter()
            .find(|category| category.fallback && category.enabled)
    }

    pub fn category_by_name(&self, name: &str) -> Option<&Category> {
        self.categories
            .iter()
            .find(|category| category.name.eq_ignore_ascii_case(name))
    }

    /// Indice de la categoria que ya usa esa extension, si hay otra.
    pub fn extension_owner(&self, extension: &str, except: usize) -> Option<usize> {
        self.categories
            .iter()
            .enumerate()
            .find(|(index, category)| {
                *index != except && category.extensions.iter().any(|ext| ext == extension)
            })
            .map(|(index, _)| index)
    }

    pub fn category_color(&self, name: &str) -> Option<String> {
        self.category_by_name(name)
            .and_then(|category| category.color.clone())
    }

    /// Copia de los ajustes que se aplica a una raiz concreta. Si esa raiz no
    /// define sobrescrituras, devuelve los ajustes globales.
    pub fn effective_for(&self, root: &Path) -> Settings {
        let mut effective = self.clone();
        let Some(entry) = self.roots.iter().find(|entry| entry.path == root) else {
            return effective;
        };
        let overrides = &entry.overrides;

        if let Some(subfolders) = &overrides.subfolders {
            effective.subfolders = subfolders.clone();
        }
        if let Some(categories) = &overrides.categories {
            effective.categories = categories.clone();
            normalize_categories(&mut effective.categories);
        }
        if let Some(rules) = &overrides.rules {
            effective.rules = rules.clone();
        }
        effective
            .watch
            .exclude_files
            .extend(overrides.exclude_files.iter().cloned());
        effective
            .watch
            .exclude_dirs
            .extend(overrides.exclude_dirs.iter().cloned());

        effective
    }

    pub fn load() -> Result<Settings, Error> {
        let path = config_path()?;

        if !path.exists() {
            return Ok(Settings::default());
        }

        let text = fs::read_to_string(&path).map_err(|err| Error::io(&path, err))?;
        let declared = declared_version(&text);
        let settings = Self::from_toml(&text).map_err(|err| Error::config(&path, err))?;

        // La copia de version es permanente: si ya existe, no se toca. Se crea
        // antes de que cualquier guardado normal pueda reemplazar el original.
        if declared < CONFIG_VERSION {
            permanent_version_backup(&path, declared)?;
        }

        Ok(settings)
    }

    pub fn read_from(path: &Path) -> Result<Settings, Error> {
        let text = fs::read_to_string(path).map_err(|err| Error::io(path, err))?;
        Self::from_toml(&text).map_err(|err| Error::config(path, err))
    }

    pub fn from_toml(text: &str) -> Result<Settings, String> {
        let mut settings: Settings = toml::from_str(text).map_err(|err| err.to_string())?;
        if settings.version > CONFIG_VERSION {
            return Err(format!(
                "config version {} is newer than supported version {CONFIG_VERSION}; update Smart Organizer",
                settings.version
            ));
        }
        settings.normalize();
        Ok(settings)
    }

    pub fn to_toml(&self) -> Result<String, Error> {
        let path = config_path()?;
        toml::to_string_pretty(self).map_err(|err| Error::config(&path, err.to_string()))
    }

    pub fn save(&self) -> Result<(), Error> {
        let dir = config_dir()?;
        fs::create_dir_all(&dir).map_err(|err| Error::io(&dir, err))?;
        self.save_to(&dir.join("config.toml"))
    }

    /// Guardado atomico con copia previa. La copia se hace antes y no mueve el
    /// original: si el reemplazo falla, `config.toml` sigue siendo el anterior.
    fn save_to(&self, path: &Path) -> Result<(), Error> {
        let text =
            toml::to_string_pretty(self).map_err(|err| Error::config(path, err.to_string()))?;

        if path.exists() {
            copy_with_sync(path, &append_suffix(path, ".bak"))?;
        }

        atomic_write(path, text.as_bytes())
    }

    pub fn export_to(&self, path: &Path) -> Result<(), Error> {
        let text =
            toml::to_string_pretty(self).map_err(|err| Error::config(path, err.to_string()))?;
        write_text_atomic(path, &text)
    }

    /// Lee una configuracion de otro sitio sin tocar la actual. Las raices
    /// protegidas o relativas se descartan antes de devolverla.
    pub fn import_from(path: &Path) -> Result<Settings, Error> {
        let mut settings = Self::read_from(path)?;
        settings
            .roots
            .retain(|root| root.path.is_absolute() && !is_protected_root(&root.path));
        Ok(settings)
    }
}

pub fn normalize_extension(ext: &str) -> String {
    let trimmed = ext.trim().trim_start_matches('*').trim_start_matches('.');
    if trimmed.is_empty() {
        return String::new();
    }
    format!(".{}", trimmed.to_ascii_lowercase())
}

fn normalize_rules(rules: &mut [Rule]) {
    for rule in rules {
        if rule.conditions.is_empty() {
            if let Some(matcher) = rule.legacy_matcher.take() {
                rule.conditions.push(matcher.to_condition());
            }
        }
        rule.legacy_matcher = None;
        rule.name = rule.name.trim().to_string();

        for condition in &mut rule.conditions {
            if let Condition::Extension { value } = condition {
                *value = normalize_extension(value);
            }
        }

        let invalid_action = match &rule.action {
            // Renombrar en el mismo sitio puede volver a aplicarse a su propio
            // resultado en cada pasada. Se conserva al leer, pero no se ejecuta.
            crate::config::Action::Rename { .. } => true,
            crate::config::Action::MoveTo { value } | crate::config::Action::CopyTo { value } => {
                value.trim().is_empty()
            }
            crate::config::Action::MoveRename { folder, template }
            | crate::config::Action::CopyRename { folder, template } => {
                folder.trim().is_empty() || template.trim().is_empty()
            }
            crate::config::Action::Trash | crate::config::Action::Ignore => false,
        };
        let invalid_condition = rule.conditions.iter().any(|condition| match condition {
            Condition::Extension { value }
            | Condition::Glob { value }
            | Condition::NameContains { value }
            | Condition::NameStartsWith { value }
            | Condition::NameEndsWith { value }
            | Condition::SourceFolder { value }
            | Condition::Regex { value } => value.trim().is_empty(),
            Condition::MinSize { .. }
            | Condition::MaxSize { .. }
            | Condition::OlderThan { .. }
            | Condition::NewerThan { .. }
            | Condition::CreatedOlderThan { .. }
            | Condition::CreatedNewerThan { .. }
            | Condition::RealType { .. } => false,
        });
        if rule.name.is_empty() || invalid_action || invalid_condition {
            rule.enabled = false;
        }

        #[cfg(feature = "regex-rules")]
        if rule.conditions.iter().any(|condition| {
            matches!(
                condition,
                Condition::Regex { value } if regex_lite::Regex::new(value).is_err()
            )
        }) {
            rule.enabled = false;
        }
    }
}

fn normalize_categories(categories: &mut Vec<Category>) {
    if categories.is_empty() {
        *categories = default_categories();
    }

    for category in categories.iter_mut() {
        category.folder = sanitize_folder(&category.folder);
        category.extensions = category
            .extensions
            .iter()
            .map(|ext| normalize_extension(ext))
            .filter(|ext| !ext.is_empty())
            .collect();
        category.extensions.sort();
        category.extensions.dedup();
        category.color = category.color.as_deref().and_then(normalize_color);
    }

    // Una sola categoria de reserva. En la version 1 era la que se llamaba Otros.
    let fallback = categories
        .iter()
        .position(|category| category.fallback)
        .or_else(|| {
            categories.iter().position(|category| {
                category.name.eq_ignore_ascii_case("Otros")
                    || category.folder.eq_ignore_ascii_case("Otros")
            })
        });

    match fallback {
        Some(index) => {
            for (position, category) in categories.iter_mut().enumerate() {
                category.fallback = position == index;
            }
        }
        None => {
            let mut category = Category::new("Otros", "Otros");
            category.fallback = true;
            categories.push(category);
        }
    }
}

fn clean_patterns(values: &[String]) -> Vec<String> {
    let mut out: Vec<String> = values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

pub fn normalize_color(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('#');
    if trimmed.len() != 6 || !trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{}", trimmed.to_ascii_lowercase()))
}

/// La carpeta se usa dentro de la raiz elegida: nunca puede traer separadores
/// ni componentes que suban de nivel.
pub fn sanitize_folder(folder: &str) -> String {
    let cleaned: String = folder
        .trim()
        .chars()
        .map(|ch| {
            if r#"<>:"/\|?*"#.contains(ch) || ch.is_control() {
                '-'
            } else {
                ch
            }
        })
        .collect();
    let mut trimmed = cleaned.trim().trim_end_matches(['.', ' ']).to_string();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        trimmed = "Otros".to_string();
    }
    if reserved_name(trimmed.split('.').next().unwrap_or(&trimmed)) {
        trimmed.insert(0, '_');
    }
    if trimmed.chars().count() > 120 {
        trimmed = trimmed.chars().take(120).collect();
        trimmed = trimmed.trim_end_matches(['.', ' ']).to_string();
    }
    trimmed
}

fn reserved_name(stem: &str) -> bool {
    matches!(
        stem.trim().to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderProblem {
    Empty,
    Invalid,
    Reserved,
    Trailing,
    TooLong,
}

/// Comprueba un nombre de carpeta antes de guardarlo. La unicidad se comprueba
/// aparte, contra la lista de categorias.
pub fn folder_problem(name: &str) -> Option<FolderProblem> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return Some(FolderProblem::Empty);
    }
    if trimmed.chars().count() > 120 {
        return Some(FolderProblem::TooLong);
    }
    if trimmed.ends_with(' ') || trimmed.ends_with('.') {
        return Some(FolderProblem::Trailing);
    }
    if trimmed
        .chars()
        .any(|ch| r#"<>:"/\|?*"#.contains(ch) || ch.is_control())
    {
        return Some(FolderProblem::Invalid);
    }

    let stem = trimmed.split('.').next().unwrap_or(trimmed);
    if reserved_name(stem) {
        return Some(FolderProblem::Reserved);
    }

    None
}

/// Une una ruta relativa a la raiz sin permitir que se salga de ella.
pub fn resolve_within(root: &Path, value: &str) -> PathBuf {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return root.to_path_buf();
    }

    let path = Path::new(trimmed);
    let relative = path.strip_prefix(root).unwrap_or(path);

    let mut out = root.to_path_buf();
    for component in relative.components() {
        if let Component::Normal(part) = component {
            let part = sanitize_folder(&part.to_string_lossy());
            if !part.is_empty() {
                out.push(part);
            }
        }
    }
    out
}

pub fn unaccent(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        let replaced = match ch {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'Á' | 'À' | 'Ä' | 'Â' => 'A',
            'É' | 'È' | 'Ë' | 'Ê' => 'E',
            'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
            'Ó' | 'Ò' | 'Ö' | 'Ô' => 'O',
            'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
            other => other,
        };
        out.push(replaced);
    }

    if out == text { None } else { Some(out) }
}

pub fn config_dir() -> Result<PathBuf, Error> {
    if let Some(dir) = portable_dir() {
        return Ok(dir);
    }

    dirs::config_dir()
        .map(|base| base.join("smart-organizer"))
        .ok_or(Error::ConfigDirMissing)
}

pub fn portable_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();
    if dir.join("portable").exists() {
        Some(dir)
    } else {
        None
    }
}

pub fn config_path() -> Result<PathBuf, Error> {
    Ok(config_dir()?.join("config.toml"))
}

/// Nombre del mutex de instancia unica. Lleva la carpeta de datos para que una
/// copia portable y una instalada puedan convivir sin bloquearse entre si.
pub fn instance_name(kind: &str) -> String {
    let dir = config_dir()
        .map(|path| path.to_string_lossy().to_lowercase())
        .unwrap_or_else(|_| "smart-organizer".to_string());
    let hash = blake3::hash(dir.as_bytes()).to_hex();
    format!("smart-organizer-{}-{kind}", &hash[..16])
}

/// Convierte rutas soltadas en carpetas aceptadas. Un archivo suelto aporta su
/// carpeta contenedora; las rutas invalidas se devuelven para avisar una sola vez.
pub fn dropped_roots(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();

    for path in paths {
        let candidate = if path.is_dir() {
            Some(path.as_path())
        } else if path.is_file() {
            path.parent()
        } else {
            None
        };

        let Some(candidate) = candidate else {
            rejected.push(path.clone());
            continue;
        };

        match accept_root(candidate) {
            Ok(canonical) => {
                if !accepted.contains(&canonical) {
                    accepted.push(canonical);
                }
            }
            Err(_) => rejected.push(path.clone()),
        }
    }

    (accepted, rejected)
}

pub fn journal_path() -> Result<PathBuf, Error> {
    Ok(config_dir()?.join("journal.jsonl"))
}

fn watch_stop_path() -> Result<PathBuf, Error> {
    Ok(config_dir()?.join("watch.stop"))
}

/// Pide al vigilante, aunque lo haya arrancado el inicio de Windows, que salga
/// en la siguiente vuelta del bucle. El archivo se borra al arrancar de nuevo.
pub fn request_watch_stop() -> Result<(), Error> {
    let path = watch_stop_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
    }
    fs::write(&path, b"stop").map_err(|err| Error::io(&path, err))
}

pub fn clear_watch_stop() -> Result<(), Error> {
    let path = watch_stop_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(Error::io(&path, err)),
    }
}

pub fn watch_stop_requested() -> bool {
    watch_stop_path().is_ok_and(|path| path.exists())
}

/// Registra o quita el vigilante oculto del arranque de Windows.
///
/// Escribe solo en HKCU, nunca en la configuracion de la maquina. Si el
/// registro falla, la app sigue funcionando y muestra el error; no se toca
/// ningun archivo del usuario.
#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), Error> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const VALUE: &str = "SmartOrganizer";

    let current = std::env::current_exe().map_err(|err| Error::Autostart {
        detail: err.to_string(),
    })?;
    let command = format!("\"{}\" --watch --hidden", current.display());

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu
        .create_subkey(RUN_KEY)
        .map_err(|err| Error::Autostart {
            detail: err.to_string(),
        })?;

    if enabled {
        run.set_value(VALUE, &command)
            .map_err(|err| Error::Autostart {
                detail: err.to_string(),
            })
    } else {
        match run.delete_value(VALUE) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(Error::Autostart {
                detail: err.to_string(),
            }),
        }
    }
}

#[cfg(not(windows))]
pub fn set_autostart(_enabled: bool) -> Result<(), Error> {
    Ok(())
}

/// Acepta una carpeta como raiz vigilada: la canonicaliza y rechaza las del sistema.
pub fn accept_root(path: &Path) -> Result<PathBuf, Error> {
    let canonical = dunce::canonicalize(path).map_err(|err| Error::io(path, err))?;

    if is_protected_root(&canonical) {
        return Err(Error::ProtectedFolder { path: canonical });
    }

    Ok(canonical)
}

fn absolute_path(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }

    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(std::path::MAIN_SEPARATOR_STR),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(name) => out.push(name),
        }
    }

    out.parent()?;
    Some(out)
}

fn same_path(first: &Path, second: &Path) -> bool {
    if cfg!(windows) {
        first
            .to_string_lossy()
            .eq_ignore_ascii_case(&second.to_string_lossy())
    } else {
        first == second
    }
}

pub fn is_protected_root(path: &Path) -> bool {
    let mut protected: Vec<PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from);
        let program_files_x86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
        let program_data = std::env::var_os("ProgramData").map(PathBuf::from);
        let windir = std::env::var_os("WINDIR").map(PathBuf::from);

        for dir in [windir, program_files, program_files_x86, program_data]
            .into_iter()
            .flatten()
        {
            protected.push(dir);
        }
    }

    #[cfg(not(windows))]
    {
        for dir in [
            "/usr", "/etc", "/bin", "/sbin", "/lib", "/boot", "/dev", "/proc", "/sys", "/var",
        ] {
            protected.push(PathBuf::from(dir));
        }
    }

    // El perfil completo del usuario incluye AppData y configuracion: se permite
    // una carpeta dentro, no el perfil entero ni sus carpetas padre.
    if let Some(home) = dirs::home_dir() {
        if home.parent().is_some() && (path == home || home.starts_with(path)) {
            return true;
        }
    }

    // Una raiz por encima de una carpeta del sistema tambien se rechaza: organizar
    // C:\ entero o C:\Users no es lo que pidio nadie.
    for dir in &protected {
        if path.starts_with(dir) || dir.starts_with(path) {
            return true;
        }
    }

    // La raiz de un disco tampoco.
    path.parent().is_none()
}

/// Añade un sufijo al nombre completo, sin tocar la extension:
/// `config.toml` + `.bak` = `config.toml.bak`.
fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

fn declared_version(text: &str) -> u32 {
    let Ok(value) = toml::from_str::<toml::Value>(text) else {
        return 0;
    };
    value
        .get("version")
        .and_then(toml::Value::as_integer)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0)
}

/// Copia permanente de la configuracion anterior a la migracion. Se crea una
/// sola vez y no se sobrescribe: es evidencia de como estaba el archivo.
fn permanent_version_backup(path: &Path, version: u32) -> Result<(), Error> {
    let backup = append_suffix(path, &format!(".v{version}.bak"));
    if backup.exists() {
        return Ok(());
    }
    copy_new_with_sync(path, &backup)
}

fn copy_with_sync(from: &Path, to: &Path) -> Result<(), Error> {
    let mut source = fs::File::open(from).map_err(|err| Error::io(from, err))?;
    let mut target = fs::File::create(to).map_err(|err| Error::io(to, err))?;
    std::io::copy(&mut source, &mut target).map_err(|err| Error::io(to, err))?;
    target.sync_all().map_err(|err| Error::io(to, err))
}

fn copy_new_with_sync(from: &Path, to: &Path) -> Result<(), Error> {
    let mut source = fs::File::open(from).map_err(|err| Error::io(from, err))?;
    let mut target = match fs::OpenOptions::new().write(true).create_new(true).open(to) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
        Err(err) => return Err(Error::io(to, err)),
    };
    std::io::copy(&mut source, &mut target).map_err(|err| Error::io(to, err))?;
    target.sync_all().map_err(|err| Error::io(to, err))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let tmp = append_suffix(path, ".tmp");

    {
        let mut file = fs::File::create(&tmp).map_err(|err| Error::io(&tmp, err))?;
        file.write_all(bytes).map_err(|err| Error::io(&tmp, err))?;
        file.sync_all().map_err(|err| Error::io(&tmp, err))?;
    }

    // El original no se mueve: se reemplaza en una sola operacion. Si algo falla,
    // `config.toml` sigue intacto y solo queda un temporal reconocible.
    #[cfg(windows)]
    let result = {
        use std::os::windows::ffi::OsStrExt;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
        }

        const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
        const MOVEFILE_WRITE_THROUGH: u32 = 0x8;

        let from: Vec<u16> = tmp.as_os_str().encode_wide().chain([0]).collect();
        let to: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let ok = unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if ok == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    };

    #[cfg(not(windows))]
    let result = fs::rename(&tmp, path);

    result.map_err(|err| {
        let _ = fs::remove_file(&tmp);
        Error::io(path, err)
    })
}

pub fn write_text_atomic(path: &Path, text: &str) -> Result<(), Error> {
    atomic_write(path, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_keeps_the_previous_file_as_a_copy() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let mut settings = Settings {
            accent: "#2563eb".to_string(),
            ..Settings::default()
        };
        settings.save_to(&path).expect("first save");
        let first = fs::read_to_string(&path).expect("first read");

        settings.accent = "#dc2626".to_string();
        settings.save_to(&path).expect("second save");
        let second = fs::read_to_string(&path).expect("second read");

        assert!(second.contains("#dc2626"));
        assert_eq!(
            fs::read_to_string(dir.path().join("config.toml.bak")).expect("backup"),
            first
        );
        assert!(!dir.path().join("config.toml.tmp").exists());
    }

    #[test]
    fn a_failed_save_keeps_the_original_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let settings = Settings::default();
        settings.save_to(&path).expect("first save");
        let original = fs::read_to_string(&path).expect("read original");

        // Un directorio con el nombre del temporal hace fallar la escritura.
        fs::create_dir(dir.path().join("config.toml.tmp")).expect("block tmp");
        let changed = Settings {
            accent: "#dc2626".to_string(),
            ..Settings::default()
        };
        assert!(changed.save_to(&path).is_err());

        assert_eq!(
            fs::read_to_string(&path).expect("read after failure"),
            original
        );
        assert!(dir.path().join("config.toml.bak").exists());
    }

    #[test]
    fn version_backups_are_permanent_and_never_overwritten() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        fs::write(&path, "version = 1\naccent = \"#111111\"\n").expect("write");

        permanent_version_backup(&path, 1).expect("first backup");
        fs::write(&path, "version = 1\naccent = \"#222222\"\n").expect("rewrite");
        permanent_version_backup(&path, 1).expect("second backup");

        let backup =
            fs::read_to_string(dir.path().join("config.toml.v1.bak")).expect("version backup");
        assert!(backup.contains("#111111"));
        assert!(!backup.contains("#222222"));
    }

    #[test]
    fn a_missing_version_is_treated_as_zero() {
        assert_eq!(declared_version("accent = \"#111111\"\n"), 0);
        assert_eq!(declared_version("version = 3\n"), 3);
    }
}
