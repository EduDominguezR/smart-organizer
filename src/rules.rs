//! Motor de reglas: decide el destino de cada archivo.
//!
//! Orden: archivos recientes, reglas personalizadas en su orden, categoria por
//! extension y categoria de reserva. Una regla puede combinar varias condiciones
//! con Y (todas) o con O (alguna). La primera regla que coincide gana.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use globset::{GlobBuilder, GlobMatcher};
use time::OffsetDateTime;

use crate::config::{self, Action, Category, Condition, DateSource, Rule, RuleSubfolder, Settings};
use crate::scanner::{self, Candidate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leave {
    Recent,
    NoCategory,
    AlreadyThere,
    Ignored,
}

#[derive(Debug, Clone)]
pub enum Target {
    Move {
        dest: PathBuf,
        name: String,
        category: String,
    },
    Copy {
        dest: PathBuf,
        name: String,
        category: String,
    },
    Trash {
        category: String,
    },
    Leave(Leave),
}

impl Target {
    pub fn category(&self) -> Option<&str> {
        match self {
            Self::Move { category, .. }
            | Self::Copy { category, .. }
            | Self::Trash { category } => Some(category),
            Self::Leave(_) => None,
        }
    }
}

pub struct Engine<'a> {
    settings: &'a Settings,
    globs: Vec<Vec<Option<GlobMatcher>>>,
    real_type_checks: bool,
    #[cfg(feature = "regex-rules")]
    regexes: Vec<Vec<Option<regex_lite::Regex>>>,
}

impl<'a> Engine<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        let globs = settings
            .rules
            .iter()
            .map(|rule| {
                rule.conditions
                    .iter()
                    .map(|condition| match condition {
                        Condition::Glob { value } => compile_glob(value),
                        _ => None,
                    })
                    .collect()
            })
            .collect();

        #[cfg(feature = "regex-rules")]
        let regexes = settings
            .rules
            .iter()
            .map(|rule| {
                rule.conditions
                    .iter()
                    .map(|condition| match condition {
                        Condition::Regex { value } => regex_lite::Regex::new(value).ok(),
                        _ => None,
                    })
                    .collect()
            })
            .collect();

        Self {
            settings,
            globs,
            real_type_checks: true,
            #[cfg(feature = "regex-rules")]
            regexes,
        }
    }

    /// Motor para el probador: no lee archivos para adivinar el tipo real.
    pub fn preview(settings: &'a Settings) -> Self {
        let mut engine = Self::new(settings);
        engine.real_type_checks = false;
        engine
    }

    pub fn decide(&self, candidate: &Candidate, now: OffsetDateTime, counter: usize) -> Target {
        if scanner::is_too_recent(candidate, self.settings.watch.recent_guard_minutes) {
            return Target::Leave(Leave::Recent);
        }

        for (index, rule) in self.settings.rules.iter().enumerate() {
            if !rule.enabled || !self.matches(index, rule, candidate, now) {
                continue;
            }

            let category = rule.name.clone();
            let extra = rule_subfolder(rule, candidate, now);
            let place =
                |folder: &str| config::resolve_within(&candidate.root, folder).join(extra.clone());

            let target = match &rule.action {
                Action::MoveTo { value } => Target::Move {
                    dest: place(value),
                    name: candidate.name.clone(),
                    category,
                },
                Action::CopyTo { value } => Target::Copy {
                    dest: place(value),
                    name: candidate.name.clone(),
                    category,
                },
                Action::Rename { .. } => return Target::Leave(Leave::Ignored),
                Action::MoveRename { folder, template } => Target::Move {
                    dest: place(folder),
                    name: expand_template(
                        template,
                        candidate,
                        &rule.name,
                        counter,
                        rule.subfolder.date_source,
                        now,
                    ),
                    category,
                },
                Action::CopyRename { folder, template } => Target::Copy {
                    dest: place(folder),
                    name: expand_template(
                        template,
                        candidate,
                        &rule.name,
                        counter,
                        rule.subfolder.date_source,
                        now,
                    ),
                    category,
                },
                Action::Trash => Target::Trash { category },
                Action::Ignore => return Target::Leave(Leave::Ignored),
            };

            // Si el archivo ya esta donde lo dejaria la regla, no se toca. Sin
            // esto, cada pasada crearia "nombre (1)" y llenaria la carpeta.
            if let Target::Move { dest, name, .. } | Target::Copy { dest, name, .. } = &target {
                if Some(dest.as_path()) == candidate.path.parent() && name == &candidate.name {
                    return Target::Leave(Leave::AlreadyThere);
                }
            }
            // Mover y renombrar dentro de la misma carpeta se aplicaria otra vez
            // sobre su propio resultado. Se deja quieto y se usa una carpeta destino.
            if let Target::Move { dest, name, .. } = &target {
                if Some(dest.as_path()) == candidate.path.parent() && name != &candidate.name {
                    return Target::Leave(Leave::Ignored);
                }
            }
            // Una copia no se repite en cada pasada: si ya existe el destino, el
            // original se queda quieto hasta que la persona borre la copia.
            if let Target::Copy { dest, name, .. } = &target {
                if dest.join(name).exists() {
                    return Target::Leave(Leave::AlreadyThere);
                }
            }
            return target;
        }

        let Some(category) = self.category_for(candidate) else {
            return Target::Leave(Leave::NoCategory);
        };

        let base = self.settings.category_folder(&candidate.root, category);
        let dest = base.join(category_subfolder(
            &self.settings.subfolders,
            candidate,
            now,
        ));

        if Some(dest.as_path()) == candidate.path.parent() {
            return Target::Leave(Leave::AlreadyThere);
        }

        Target::Move {
            dest,
            name: candidate.name.clone(),
            category: category.name.clone(),
        }
    }

    fn category_for(&self, candidate: &Candidate) -> Option<&Category> {
        self.settings
            .categories
            .iter()
            .find(|category| {
                category.enabled
                    && !candidate.ext.is_empty()
                    && category.extensions.contains(&candidate.ext)
            })
            .or_else(|| self.settings.fallback_category())
    }

    fn matches(
        &self,
        index: usize,
        rule: &Rule,
        candidate: &Candidate,
        now: OffsetDateTime,
    ) -> bool {
        if rule.conditions.is_empty() {
            return false;
        }

        let mut result = rule.match_all;
        for (position, condition) in rule.conditions.iter().enumerate() {
            let matched = self.condition_matches(index, position, condition, candidate, now);
            if rule.match_all {
                if !matched {
                    return false;
                }
            } else if matched {
                return true;
            }
            result = matched;
        }

        result
    }

    fn condition_matches(
        &self,
        rule_index: usize,
        condition_index: usize,
        condition: &Condition,
        candidate: &Candidate,
        now: OffsetDateTime,
    ) -> bool {
        match condition {
            Condition::Extension { value } => candidate.ext == *value,
            Condition::Glob { value: _ } => self
                .globs
                .get(rule_index)
                .and_then(|list| list.get(condition_index))
                .and_then(Option::as_ref)
                .is_some_and(|glob| glob.is_match(&candidate.name)),
            Condition::NameContains { value } => contains(&candidate.name, value),
            Condition::NameStartsWith { value } => starts_with(&candidate.name, value),
            Condition::NameEndsWith { value } => ends_with(&candidate.name, value),
            Condition::MinSize { value } => candidate.size >= *value,
            Condition::MaxSize { value } => candidate.size <= *value,
            Condition::OlderThan { value } => {
                age_days(candidate.modified, now) >= u64::from(*value)
            }
            Condition::NewerThan { value } => {
                age_days(candidate.modified, now) <= u64::from(*value)
            }
            Condition::CreatedOlderThan { value } => candidate
                .created
                .is_some_and(|created| age_days(created, now) >= u64::from(*value)),
            Condition::CreatedNewerThan { value } => candidate
                .created
                .is_some_and(|created| age_days(created, now) <= u64::from(*value)),
            Condition::RealType { value } => {
                self.real_type_checks && real_type_matches(&candidate.path, value)
            }
            Condition::SourceFolder { value } => {
                let folder = candidate
                    .path
                    .parent()
                    .and_then(|parent| parent.strip_prefix(&candidate.root).ok())
                    .map(|parent| parent.to_string_lossy().replace('\\', "/").to_lowercase())
                    .unwrap_or_default();
                let wanted = value.trim().replace('\\', "/").to_lowercase();
                if wanted.contains('*') || wanted.contains('?') {
                    compile_glob(&wanted).is_some_and(|glob| glob.is_match(&folder))
                } else {
                    folder == wanted || folder.starts_with(&format!("{wanted}/"))
                }
            }
            Condition::Regex { .. } => {
                #[cfg(feature = "regex-rules")]
                {
                    self.regexes
                        .get(rule_index)
                        .and_then(|list| list.get(condition_index))
                        .and_then(Option::as_ref)
                        .is_some_and(|regex| regex.is_match(&candidate.name))
                }
                #[cfg(not(feature = "regex-rules"))]
                {
                    false
                }
            }
        }
    }
}

fn compile_glob(value: &str) -> Option<GlobMatcher> {
    GlobBuilder::new(value)
        .case_insensitive(true)
        .build()
        .ok()
        .map(|glob| glob.compile_matcher())
}

fn contains(text: &str, value: &str) -> bool {
    text.to_lowercase().contains(&value.trim().to_lowercase())
}

fn starts_with(text: &str, value: &str) -> bool {
    text.to_lowercase()
        .starts_with(&value.trim().to_lowercase())
}

fn ends_with(text: &str, value: &str) -> bool {
    text.to_lowercase().ends_with(&value.trim().to_lowercase())
}

fn age_days(stamp: SystemTime, now: OffsetDateTime) -> u64 {
    let stamp = OffsetDateTime::from(stamp);
    let age = now - stamp;
    if age.is_negative() {
        0
    } else {
        age.whole_seconds().unsigned_abs() / 86_400
    }
}

fn real_type_matches(path: &Path, value: &str) -> bool {
    let Ok(Some(kind)) = infer::get_from_path(path) else {
        return false;
    };
    let actual = match kind.matcher_type() {
        infer::MatcherType::App => "program",
        infer::MatcherType::Archive => "archive",
        infer::MatcherType::Audio => "audio",
        infer::MatcherType::Book => "book",
        infer::MatcherType::Doc => "document",
        infer::MatcherType::Font => "font",
        infer::MatcherType::Image => "image",
        infer::MatcherType::Text => "text",
        infer::MatcherType::Video => "video",
        infer::MatcherType::Custom => "custom",
    };
    actual.eq_ignore_ascii_case(value)
}

/// Fecha que usan las subcarpetas por año y mes.
pub fn date_for(source: DateSource, candidate: &Candidate, now: OffsetDateTime) -> OffsetDateTime {
    let stamp = match source {
        DateSource::Today => return now,
        DateSource::Created => candidate.created.unwrap_or(candidate.modified),
        DateSource::Modified => candidate.modified,
    };
    OffsetDateTime::from(stamp).to_offset(now.offset())
}

fn category_subfolder(
    settings: &crate::config::Subfolders,
    candidate: &Candidate,
    now: OffsetDateTime,
) -> PathBuf {
    let mut path = PathBuf::new();

    if settings.by_year_month {
        let date = date_for(settings.date_source, candidate, now);
        path.push(format!("{:04}", date.year()));
        path.push(format!("{:02}", u8::from(date.month())));
    }

    if settings.by_capture && is_capture(&candidate.name) {
        path.push("Capturas");
    } else if settings.by_type && !candidate.ext.is_empty() {
        path.push(candidate.ext.trim_start_matches('.').to_ascii_uppercase());
    }

    path
}

fn rule_subfolder(rule: &Rule, candidate: &Candidate, now: OffsetDateTime) -> PathBuf {
    let settings: &RuleSubfolder = &rule.subfolder;
    if !settings.enabled {
        return PathBuf::new();
    }

    let mut path = PathBuf::new();
    if settings.by_year_month {
        let date = date_for(settings.date_source, candidate, now);
        path.push(format!("{:04}", date.year()));
        path.push(format!("{:02}", u8::from(date.month())));
    }
    if settings.by_type && !candidate.ext.is_empty() {
        path.push(candidate.ext.trim_start_matches('.').to_ascii_uppercase());
    }
    path
}

pub fn is_capture(name: &str) -> bool {
    let lower = name.to_lowercase();
    [
        "captura",
        "screenshot",
        "screen shot",
        "snipping",
        "capture d ecran",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

pub fn expand_template(
    template: &str,
    candidate: &Candidate,
    category: &str,
    counter: usize,
    date_source: DateSource,
    now: OffsetDateTime,
) -> String {
    let date = date_for(date_source, candidate, now);
    let year = format!("{:04}", date.year());
    let month = format!("{:02}", u8::from(date.month()));
    let day = format!("{:02}", date.day());
    let original = candidate.stem();

    let mut name = template
        .replace("{nombre}", &original)
        .replace("{original}", &original)
        .replace("{ext}", candidate.ext.trim_start_matches('.'))
        .replace("{fecha}", &format!("{year}-{month}-{day}"))
        .replace("{año}", &year)
        .replace("{ano}", &year)
        .replace("{mes}", &month)
        .replace("{dia}", &day)
        .replace("{día}", &day)
        .replace("{contador}", &counter.to_string())
        .replace("{categoria}", category)
        .replace("{categoría}", category);

    if Path::new(&name).extension().is_none() && !candidate.ext.is_empty() {
        name.push_str(&candidate.ext);
    }

    sanitize_file_name(&name)
}

fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|ch| if r#"<>:"/\|?*"#.contains(ch) { '-' } else { ch })
        .collect();

    let mut trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        trimmed = "archivo".to_string();
    }

    let stem = trimmed
        .split('.')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_uppercase();
    if matches!(
        stem.as_str(),
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
    ) {
        trimmed.insert(0, '_');
    }

    if trimmed.chars().count() > 180 {
        trimmed = trimmed.chars().take(180).collect();
    }
    trimmed
}

pub struct TestResult {
    pub source: Option<String>,
    pub target: Target,
}

/// Destino que tendria un nombre de archivo con las reglas actuales. Se usa en
/// la vista previa de la interfaz; no toca el disco.
pub fn preview_destination(settings: &Settings, name: &str) -> Target {
    test(settings, None, name, 0, 0).target
}

/// Igual que `preview_destination`, pero para una carpeta concreta cuando se
/// esta editando una regla con ambito propio.
pub fn preview_destination_in(settings: &Settings, root: &Path, name: &str) -> Target {
    test(settings, Some(root), name, 0, 0).target
}

/// Prueba una regla con un nombre, un tamaño y una antigüedad, sin tocar el disco.
pub fn test(
    settings: &Settings,
    root: Option<&Path>,
    name: &str,
    size: u64,
    age_days: u32,
) -> TestResult {
    let root = root
        .map(Path::to_path_buf)
        .or_else(|| settings.active_roots().first().cloned())
        .unwrap_or_default();
    let path = root.join(name);
    let extension = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy().to_ascii_lowercase()))
        .unwrap_or_default();
    let age = Duration::from_secs(u64::from(age_days) * 86_400);
    let modified = SystemTime::now()
        .checked_sub(age)
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let candidate = Candidate {
        path,
        root,
        size,
        modified,
        created: Some(modified),
        name: name.to_string(),
        ext: extension,
    };

    let now = OffsetDateTime::now_utc();
    let engine = Engine::preview(settings);
    let target = engine.decide(&candidate, now, 1);
    let source = target.category().map(str::to_string);

    TestResult { source, target }
}
