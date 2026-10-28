//! Ventana principal: barra superior, pestanas y estado.
//!
//! La interfaz no toca los archivos. Manda trabajo a un hilo de fondo y recibe
//! resultados; mientras tanto el repintado sigue siendo bajo demanda.

pub mod ajustes;
pub mod duplicados;
pub mod historial;
pub mod icon;
pub mod inicio;
pub mod organizar;
pub mod reglas;
pub mod theme;
#[cfg(windows)]
mod tray;
pub mod vista_previa;
pub mod widgets;

use std::path::{Path, PathBuf};
use std::process::{Child, Command as Process, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{
    self, Category, Condition, Density, Profile, Root, Rule, Settings, TextSize, Theme,
};
use crate::error::Error;
use crate::i18n::{self, Key, Lang};
use crate::journal::{BatchSummary, Totals, UndoReport};
use crate::mover;
use crate::organize::{self, Plan, Summary};
use eframe::egui::{self, Align, Color32, Layout, Margin, RichText};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Inicio,
    Vista,
    Historial,
    Ajustes,
    Organizar,
}

impl Tab {
    fn label(self, lang: Lang) -> &'static str {
        match self {
            Tab::Inicio => i18n::t(lang, Key::TabHome),
            Tab::Vista => i18n::t(lang, Key::TabPreview),
            Tab::Historial => i18n::t(lang, Key::TabHistory),
            Tab::Ajustes => i18n::t(lang, Key::TabSettings),
            Tab::Organizar => i18n::t(lang, Key::TabOrganize),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Carpetas,
    Estilo,
    Reglas,
    Vigilancia,
    Duplicados,
    Apariencia,
    Avanzado,
}

pub struct Notice {
    pub text: String,
    pub error: bool,
}

impl Notice {
    pub fn info(text: String) -> Self {
        Self { text, error: false }
    }

    pub fn error(text: String) -> Self {
        Self { text, error: true }
    }
}

pub enum Confirm {
    Move {
        files: usize,
        trash: usize,
    },
    Undo {
        batch: Option<String>,
        files: usize,
    },
    Reset,
    FolderRename {
        scope: Scope,
        old: String,
        new: String,
        entries: usize,
    },
    Duplicates {
        files: usize,
        bytes: u64,
    },
    Import {
        settings: Box<Settings>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Global,
    Root(usize),
}

pub struct FolderEdit {
    pub scope: Scope,
    pub index: usize,
    pub value: String,
    pub error: Option<String>,
}

pub enum UndoChange {
    Folder {
        root: Root,
        exclude_files: String,
        exclude_dirs: String,
    },
    Groups {
        scope: Scope,
        categories: Vec<Category>,
        excludes: Vec<Vec<String>>,
    },
    Settings {
        previous: Box<Settings>,
    },
    Rules {
        scope: Scope,
        rules: Vec<Rule>,
    },
}

pub struct TesterDraft {
    pub name: String,
    pub size_kb: String,
    pub days: String,
}

impl Default for TesterDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            size_kb: "0".to_string(),
            days: "0".to_string(),
        }
    }
}

pub struct RuleDraft {
    pub editing: Option<u64>,
    pub enabled: bool,
    pub name: String,
    pub match_all: bool,
    pub conditions: Vec<Condition>,
    pub simple_kind: usize,
    pub simple_value: String,
    pub condition_kind: usize,
    pub condition_value: String,
    pub condition_unit: usize,
    pub action: usize,
    pub target: String,
    pub template: String,
    pub sub_enabled: bool,
    pub sub_year_month: bool,
    pub sub_type: bool,
}

impl Default for RuleDraft {
    fn default() -> Self {
        Self {
            editing: None,
            enabled: true,
            name: String::new(),
            match_all: true,
            conditions: Vec::new(),
            simple_kind: 2,
            simple_value: String::new(),
            condition_kind: 0,
            condition_value: String::new(),
            condition_unit: 1,
            action: 0,
            target: String::new(),
            template: String::new(),
            sub_enabled: false,
            sub_year_month: false,
            sub_type: false,
        }
    }
}

impl RuleDraft {
    pub fn from_rule(rule: &Rule) -> Self {
        let (action, target, template) = match &rule.action {
            crate::config::Action::MoveTo { value } => (0, value.clone(), String::new()),
            crate::config::Action::Trash => (1, String::new(), String::new()),
            crate::config::Action::Rename { value } => (2, value.clone(), String::new()),
            crate::config::Action::CopyTo { value } => (3, value.clone(), String::new()),
            crate::config::Action::MoveRename { folder, template } => {
                (4, folder.clone(), template.clone())
            }
            crate::config::Action::CopyRename { folder, template } => {
                (5, folder.clone(), template.clone())
            }
            crate::config::Action::Ignore => (6, String::new(), String::new()),
        };

        let (simple_kind, simple_value) = match rule.conditions.first() {
            Some(Condition::Extension { value }) => (0, value.trim_start_matches('.').to_string()),
            Some(Condition::MinSize { value }) => (5, (value / 1_048_576).to_string()),
            Some(Condition::OlderThan { value }) => (7, value.to_string()),
            Some(Condition::NameContains { value }) => (2, value.clone()),
            _ => (2, String::new()),
        };

        Self {
            editing: Some(rule.id),
            enabled: rule.enabled,
            name: rule.name.clone(),
            match_all: rule.match_all,
            conditions: rule.conditions.clone(),
            simple_kind,
            simple_value,
            action,
            target,
            template,
            sub_enabled: rule.subfolder.enabled,
            sub_year_month: rule.subfolder.by_year_month,
            sub_type: rule.subfolder.by_type,
            ..Self::default()
        }
    }
}

enum Command {
    Scan {
        settings: Settings,
        lang: Lang,
    },
    Organize {
        plan: Plan,
        stability_ms: u64,
        lang: Lang,
    },
    Undo {
        batch: Option<String>,
        lang: Lang,
    },
    History,
}

enum Reply {
    Plan(Result<Plan, String>),
    Organized(Result<Summary, String>),
    Undone(Result<UndoReport, String>),
    History(Vec<BatchSummary>, Totals),
}

struct Worker {
    commands: Sender<Command>,
    replies: Receiver<Reply>,
}

impl Worker {
    fn new(ctx: egui::Context) -> Self {
        let (commands, command_rx) = mpsc::channel::<Command>();
        let (reply_tx, replies) = mpsc::channel::<Reply>();

        thread::spawn(move || {
            crate::priority::lower_thread();
            while let Ok(command) = command_rx.recv() {
                let reply = match command {
                    Command::Scan { settings, lang } => {
                        Reply::Plan(organize::build(&settings).map_err(|err| err.describe(lang)))
                    }
                    Command::Organize {
                        plan,
                        stability_ms,
                        lang,
                    } => Reply::Organized(
                        organize::execute(&plan, stability_ms).map_err(|err| err.describe(lang)),
                    ),
                    Command::Undo { batch, lang } => {
                        let result = match batch {
                            Some(batch) => organize::undo_batch(&batch),
                            None => organize::undo_last(),
                        };
                        Reply::Undone(result.map_err(|err| err.describe(lang)))
                    }
                    Command::History => {
                        let history = organize::history().unwrap_or_default();
                        let totals = organize::totals().unwrap_or_default();
                        Reply::History(history, totals)
                    }
                };

                if reply_tx.send(reply).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });

        Self { commands, replies }
    }

    fn send(&self, command: Command) {
        // Si el hilo murio no hay nada que hacer: la proxima accion lo reintenta.
        let _ = self.commands.send(command);
    }

    fn poll(&self) -> Option<Reply> {
        self.replies.try_recv().ok()
    }
}

pub struct App {
    pub lang: Lang,
    pub settings: Settings,
    pub duplicates: duplicados::State,
    pub tab: Tab,
    pub section: SettingsSection,
    pub plan: Option<Plan>,
    pub scanning: bool,
    pub organizing: bool,
    pub history: Vec<BatchSummary>,
    pub totals: Totals,
    pub notice: Option<Notice>,
    pub confirm: Option<Confirm>,
    pub filter: String,
    pub draft_extension: String,
    pub draft_category: usize,
    pub rule_draft: RuleDraft,
    pub rule_error: Option<String>,
    pub tester: TesterDraft,
    pub scope: Scope,
    pub folder_edit: Option<FolderEdit>,
    pub undo_change: Option<UndoChange>,
    pub profile_choice: Option<Profile>,
    pub expanded_group: Option<usize>,
    pub draft_type: String,
    pub live_preview: String,
    pub onboard_step: usize,
    save_shown_at: Option<Instant>,
    accent_draft: String,
    pub temp_text: String,
    pub skip_text: String,
    pub root_exclude_files: Vec<String>,
    pub root_exclude_dirs: Vec<String>,
    pub excludes: String,
    pub file_excludes: String,
    pub watch_child: Option<Child>,
    external_watch: bool,
    /// Solo se restaura un vigilante que arranco esta ventana y no se detuvo a
    /// proposito. Un fallo externo se controla con `watch.stop`.
    watch_supervisor: bool,
    watch_attempts: Vec<Instant>,
    watch_retry_at: Option<Instant>,
    pub watch_failed: bool,
    #[cfg(windows)]
    tray: Option<tray::Tray>,
    #[cfg(windows)]
    quit_requested: bool,
    tray_organize: bool,
    worker: Worker,
    applied: Option<(Theme, String, TextSize, Density, bool)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut notice = None;
        let mut settings = match Settings::load() {
            Ok(settings) => settings,
            Err(err) => {
                notice = Some(Notice::error(err.describe(Lang::default())));
                Settings::default()
            }
        };
        if !settings.roots.is_empty() && !settings.onboarded {
            settings.onboarded = true;
        }

        let lang = if settings.language == "en" {
            Lang::En
        } else {
            Lang::default()
        };

        theme::install_system_fonts(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx, &settings);

        let excludes = settings.watch.exclude_dirs.join(", ");
        let file_excludes = settings.watch.exclude_files.join(", ");
        let temp_text = settings.temp_extensions.join(", ");
        let skip_text = settings.skip_dirs.join(", ");
        let root_exclude_files = settings
            .roots
            .iter()
            .map(|root| root.overrides.exclude_files.join(", "))
            .collect();
        let root_exclude_dirs = settings
            .roots
            .iter()
            .map(|root| root.overrides.exclude_dirs.join(", "))
            .collect();
        let accent_draft = settings.accent.clone();

        #[cfg(windows)]
        let tray = tray::Tray::new(&cc.egui_ctx, lang).ok();

        let mut app = Self {
            lang,
            applied: Some((
                settings.theme,
                settings.accent.clone(),
                settings.text_size,
                settings.density,
                cc.egui_ctx.theme() == egui::Theme::Dark,
            )),
            settings,
            duplicates: duplicados::State::default(),
            tab: Tab::Inicio,
            section: SettingsSection::General,
            plan: None,
            scanning: false,
            organizing: false,
            history: Vec::new(),
            totals: Totals::default(),
            notice,
            confirm: None,
            filter: String::new(),
            draft_extension: String::new(),
            draft_category: 0,
            rule_draft: RuleDraft::default(),
            rule_error: None,
            tester: TesterDraft::default(),
            scope: Scope::Global,
            folder_edit: None,
            undo_change: None,
            profile_choice: None,
            expanded_group: None,
            draft_type: String::new(),
            live_preview: "factura.pdf".to_string(),
            onboard_step: 0,
            save_shown_at: None,
            accent_draft,
            temp_text,
            skip_text,
            root_exclude_files,
            root_exclude_dirs,
            excludes,
            file_excludes,
            watch_child: None,
            external_watch: false,
            watch_supervisor: false,
            watch_attempts: Vec::new(),
            watch_retry_at: None,
            watch_failed: false,
            #[cfg(windows)]
            tray,
            #[cfg(windows)]
            quit_requested: false,
            tray_organize: false,
            worker: Worker::new(cc.egui_ctx.clone()),
        };

        if app.settings.roots.is_empty() {
            app.tab = Tab::Inicio;
        } else {
            app.scan();
        }

        app.refresh_history();

        if app.settings.background {
            app.start_watcher();
        }

        app
    }

    fn save(&mut self) {
        self.settings.normalize();
        match self.settings.save() {
            Ok(()) => self.save_shown_at = Some(Instant::now()),
            Err(err) => {
                self.save_shown_at = None;
                self.notice = Some(Notice::error(err.describe(self.lang)));
            }
        }
    }

    /// Carpetas conocidas que existen, no estan ya anadidas y se pueden
    /// ofrecer como atajo.
    pub fn available_shortcuts(&self) -> Vec<(Key, PathBuf)> {
        [
            (
                Key::FolderDownloads,
                crate::known_folders::path(crate::known_folders::KnownFolder::Downloads),
            ),
            (
                Key::FolderDesktop,
                crate::known_folders::path(crate::known_folders::KnownFolder::Desktop),
            ),
            (
                Key::FolderDocuments,
                crate::known_folders::path(crate::known_folders::KnownFolder::Documents),
            ),
        ]
        .into_iter()
        .filter_map(|(key, path)| {
            let path = path?;
            if !path.is_dir() {
                return None;
            }
            let path = config::accept_root(&path).ok()?;
            if self.settings.roots.iter().any(|root| root.path == path) {
                return None;
            }
            Some((key, path))
        })
        .collect()
    }

    pub fn shortcut_buttons(&mut self, ui: &mut egui::Ui, palette: &theme::Palette) {
        let shortcuts = self.available_shortcuts();
        if shortcuts.is_empty() {
            return;
        }
        widgets::small_muted(ui, palette, i18n::t(self.lang, Key::ShortcutsHint));
        ui.horizontal_wrapped(|ui| {
            for (key, path) in shortcuts {
                if widgets::ghost_button(ui, palette, &format!("+ {}", i18n::t(self.lang, key)))
                    .clicked()
                {
                    self.add_root(&path);
                }
            }
        });
    }

    /// Añade todas las carpetas soltadas de una vez. Un archivo suelto aporta
    /// su carpeta contenedora; los rechazos se resumen en un solo aviso.
    pub fn add_dropped(&mut self, paths: &[PathBuf]) {
        let (accepted, rejected) = config::dropped_roots(paths);
        let mut added = 0usize;
        for path in accepted {
            if self.settings.roots.iter().any(|root| root.path == path) {
                continue;
            }
            self.settings.roots.push(Root::new(path));
            self.root_exclude_files.push(String::new());
            self.root_exclude_dirs.push(String::new());
            added += 1;
        }

        if added > 0 {
            self.save();
            self.scan();
        }

        if added > 0 || rejected.is_empty() {
            let mut text = if added > 0 {
                i18n::drop_added(self.lang, added)
            } else {
                i18n::t(self.lang, Key::FolderAlreadyAdded).to_string()
            };
            if !rejected.is_empty() {
                text.push(' ');
                text.push_str(&i18n::drop_rejected(self.lang, rejected.len()));
            }
            self.notice = Some(if rejected.is_empty() {
                Notice::info(text)
            } else {
                Notice::error(text)
            });
        } else {
            self.notice = Some(Notice::error(i18n::drop_none(self.lang)));
        }
    }

    pub fn remove_root(&mut self, index: usize) {
        if index >= self.settings.roots.len() {
            return;
        }
        let root = self.settings.roots.remove(index);
        let exclude_files = root.overrides.exclude_files.join(", ");
        let exclude_dirs = root.overrides.exclude_dirs.join(", ");
        if index < self.root_exclude_files.len() {
            self.root_exclude_files.remove(index);
        }
        if index < self.root_exclude_dirs.len() {
            self.root_exclude_dirs.remove(index);
        }
        self.undo_change = Some(UndoChange::Folder {
            root,
            exclude_files,
            exclude_dirs,
        });
        if self.scope == Scope::Root(index) {
            self.scope = Scope::Global;
        } else if let Scope::Root(scope_index) = self.scope {
            if scope_index > index {
                self.scope = Scope::Root(scope_index - 1);
            }
        }
        self.plan = None;
        self.save();
        self.scan();
        self.notice = Some(Notice::info(
            i18n::t(self.lang, Key::FolderRemoved).to_string(),
        ));
    }

    pub fn undo_pending_change(&mut self) {
        match self.undo_change.take() {
            Some(UndoChange::Settings { previous }) => {
                self.settings = *previous;
                self.accent_draft = self.settings.accent.clone();
                self.save();
                self.scan();
            }
            Some(UndoChange::Rules { scope, rules }) => {
                self.set_scope_rules(scope, rules);
                self.save();
                self.scan();
            }
            Some(UndoChange::Folder {
                root,
                exclude_files,
                exclude_dirs,
            }) => {
                self.settings.roots.push(root);
                self.root_exclude_files.push(exclude_files);
                self.root_exclude_dirs.push(exclude_dirs);
                self.save();
                self.scan();
            }
            Some(UndoChange::Groups {
                scope,
                categories,
                excludes,
            }) => {
                self.set_scope_categories(scope, categories);
                self.restore_excludes(excludes);
                self.save();
                self.scan();
            }
            None => {}
        }
        self.notice = None;
    }

    fn snapshot_groups(&mut self, scope: Scope) {
        let Some(categories) = self.scope_categories(scope).cloned() else {
            return;
        };
        let excludes = self
            .settings
            .roots
            .iter()
            .map(|root| root.overrides.exclude_dirs.clone())
            .collect();
        self.undo_change = Some(UndoChange::Groups {
            scope,
            categories,
            excludes,
        });
    }

    fn restore_excludes(&mut self, excludes: Vec<Vec<String>>) {
        for (root, dirs) in self.settings.roots.iter_mut().zip(excludes) {
            root.overrides.exclude_dirs = dirs;
        }
        self.root_exclude_dirs = self
            .settings
            .roots
            .iter()
            .map(|root| root.overrides.exclude_dirs.join(", "))
            .collect();
    }

    pub fn delete_group(&mut self, scope: Scope, index: usize) {
        let fallback = self
            .scope_categories(scope)
            .and_then(|categories| categories.get(index))
            .is_some_and(|category| category.fallback);
        if fallback {
            self.notice = Some(Notice::info(
                i18n::t(self.lang, Key::GroupNoDelete).to_string(),
            ));
            return;
        }
        let Some(original) = self.scope_categories(scope).cloned() else {
            return;
        };
        if index >= original.len() {
            return;
        }
        self.snapshot_groups(scope);
        let folder = original[index].folder.clone();
        let mut categories = original;
        categories.remove(index);
        self.set_scope_categories(scope, categories);
        let folder_exists = self
            .scope_roots(scope)
            .iter()
            .any(|root| root.join(&folder).exists());
        if folder_exists {
            let pattern = globset::escape(&folder);
            let roots = match scope {
                Scope::Global => self.settings.roots.iter_mut().collect::<Vec<_>>(),
                Scope::Root(index) => self.settings.roots.get_mut(index).into_iter().collect(),
            };
            for root in roots {
                if !root
                    .overrides
                    .exclude_dirs
                    .iter()
                    .any(|value| value == &pattern)
                {
                    root.overrides.exclude_dirs.push(pattern.clone());
                }
            }
            self.root_exclude_dirs = self
                .settings
                .roots
                .iter()
                .map(|root| root.overrides.exclude_dirs.join(", "))
                .collect();
        }
        self.expanded_group = None;
        self.save();
        self.scan();
        self.notice = Some(Notice::info(
            i18n::t(self.lang, Key::GroupDeleted).to_string(),
        ));
    }

    pub fn add_group(&mut self, scope: Scope) {
        let Some(mut categories) = self.scope_categories(scope).cloned() else {
            return;
        };
        let mut number = categories.len() + 1;
        let mut name =
            i18n::t(self.lang, Key::GroupNewName).replace("{number}", &number.to_string());
        while categories.iter().any(|category| category.name == name) {
            number += 1;
            name = i18n::t(self.lang, Key::GroupNewName).replace("{number}", &number.to_string());
        }
        let folder = config::sanitize_folder(&name);
        categories.push(Category::new(&name, &folder));
        self.expanded_group = Some(categories.len() - 1);
        self.set_scope_categories(scope, categories);
        self.save();
    }

    pub fn move_group(&mut self, scope: Scope, index: usize, delta: isize) {
        let Some(mut categories) = self.scope_categories(scope).cloned() else {
            return;
        };
        let target = index as isize + delta;
        if target < 0 || target as usize >= categories.len() {
            return;
        }
        categories.swap(index, target as usize);
        self.expanded_group = Some(target as usize);
        self.set_scope_categories(scope, categories);
        self.save();
        self.scan();
    }

    pub fn restore_groups(&mut self, scope: Scope) {
        if self.scope_categories(scope).is_none() {
            return;
        }
        self.snapshot_groups(scope);
        self.set_scope_categories(scope, config::default_categories());
        self.expanded_group = None;
        self.save();
        self.scan();
        self.notice = Some(Notice::info(
            i18n::t(self.lang, Key::GroupRestoreDone).to_string(),
        ));
    }

    pub fn scope_categories(&self, scope: Scope) -> Option<&Vec<Category>> {
        match scope {
            Scope::Global => Some(&self.settings.categories),
            Scope::Root(index) => self
                .settings
                .roots
                .get(index)?
                .overrides
                .categories
                .as_ref(),
        }
    }

    /// Raices cuya configuracion efectiva usa este ambito.
    pub fn scope_roots(&self, scope: Scope) -> Vec<PathBuf> {
        match scope {
            Scope::Global => self.settings.active_roots(),
            Scope::Root(index) => self
                .settings
                .roots
                .get(index)
                .filter(|root| root.enabled)
                .map(|root| vec![root.path.clone()])
                .unwrap_or_default(),
        }
    }

    pub fn scope_rules(&self, scope: Scope) -> Option<&Vec<Rule>> {
        match scope {
            Scope::Global => Some(&self.settings.rules),
            Scope::Root(index) => self.settings.roots.get(index)?.overrides.rules.as_ref(),
        }
    }

    pub fn set_scope_categories(&mut self, scope: Scope, categories: Vec<Category>) {
        match scope {
            Scope::Global => self.settings.categories = categories,
            Scope::Root(index) => {
                if let Some(root) = self.settings.roots.get_mut(index) {
                    root.overrides.categories = Some(categories);
                }
            }
        }
    }

    pub fn set_scope_rules(&mut self, scope: Scope, rules: Vec<Rule>) {
        match scope {
            Scope::Global => self.settings.rules = rules,
            Scope::Root(index) => {
                if let Some(root) = self.settings.roots.get_mut(index) {
                    root.overrides.rules = Some(rules);
                }
            }
        }
    }

    pub fn scope_root(&self, scope: Scope) -> Option<&Path> {
        match scope {
            Scope::Global => None,
            Scope::Root(index) => self
                .settings
                .roots
                .get(index)
                .map(|root| root.path.as_path()),
        }
    }

    /// Ajustes que veria una raiz concreta, para el probador de reglas.
    pub fn scope_settings(&self, scope: Scope) -> Settings {
        match scope {
            Scope::Global => self.settings.clone(),
            Scope::Root(index) => self
                .settings
                .roots
                .get(index)
                .map(|root| self.settings.effective_for(&root.path))
                .unwrap_or_else(|| self.settings.clone()),
        }
    }

    pub fn add_root(&mut self, path: &Path) {
        match config::accept_root(path) {
            Ok(canonical) => {
                if self
                    .settings
                    .roots
                    .iter()
                    .any(|root| root.path == canonical)
                {
                    self.notice = Some(Notice::info(
                        i18n::t(self.lang, Key::FolderAlreadyAdded).to_string(),
                    ));
                    return;
                }
                self.settings.roots.push(Root::new(canonical));
                self.root_exclude_files.push(String::new());
                self.root_exclude_dirs.push(String::new());
                self.save();
                self.scan();
            }
            Err(err) => self.notice = Some(Notice::error(err.describe(self.lang))),
        }
    }

    pub fn export_config(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, Key::ExportConfig))
            .set_file_name("smart-organizer-config.toml")
            .add_filter("TOML", &["toml"])
            .save_file()
        else {
            return;
        };

        match self.settings.export_to(&path) {
            Ok(()) => {
                self.notice = Some(Notice::info(i18n::config_exported(self.lang, &path)));
            }
            Err(err) => self.notice = Some(Notice::error(err.describe(self.lang))),
        }
    }

    pub fn import_config(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, Key::ImportConfig))
            .add_filter("TOML", &["toml"])
            .pick_file()
        else {
            return;
        };

        match Settings::import_from(&path) {
            Ok(settings) => {
                self.confirm = Some(Confirm::Import {
                    settings: Box::new(settings),
                });
            }
            Err(err) => {
                self.notice = Some(Notice::error(format!(
                    "{} {}",
                    i18n::t(self.lang, Key::ImportInvalid),
                    err.describe(self.lang)
                )));
            }
        }
    }

    pub fn scan(&mut self) {
        if self.settings.active_roots().is_empty() {
            self.plan = None;
            return;
        }
        self.scanning = true;
        self.worker.send(Command::Scan {
            settings: self.settings.clone(),
            lang: self.lang,
        });
    }

    fn refresh_history(&mut self) {
        self.worker.send(Command::History);
    }

    fn organize_now(&mut self) {
        let Some(plan) = self.plan.take() else {
            self.scan();
            return;
        };

        if plan.is_empty() {
            self.notice = Some(Notice::info(
                i18n::t(self.lang, Key::EmptyNoPlan).to_string(),
            ));
            self.scan();
            return;
        }

        self.organizing = true;
        self.worker.send(Command::Organize {
            plan,
            stability_ms: self.settings.watch.stability_ms,
            lang: self.lang,
        });
    }

    pub fn ask_organize(&mut self) {
        let Some(plan) = &self.plan else {
            self.scan();
            return;
        };

        if plan.is_empty() {
            self.notice = Some(Notice::info(
                i18n::t(self.lang, Key::EmptyNoPlan).to_string(),
            ));
            return;
        }

        self.confirm = Some(Confirm::Move {
            files: plan.selected_count(),
            trash: plan.trash_count(),
        });
    }

    pub fn ask_undo_last(&mut self) {
        let Some(batch) = self
            .history
            .iter()
            .find(|batch| !batch.undone && batch.files > batch.trashed)
        else {
            self.notice = Some(Notice::info(
                i18n::t(self.lang, Key::NothingToUndo).to_string(),
            ));
            return;
        };

        self.confirm = Some(Confirm::Undo {
            batch: Some(batch.id.clone()),
            files: batch.files,
        });
    }

    pub fn ask_undo_batch(&mut self, batch: &BatchSummary) {
        self.confirm = Some(Confirm::Undo {
            batch: Some(batch.id.clone()),
            files: batch.files,
        });
    }

    #[cfg(windows)]
    fn poll_tray(&mut self, ctx: &egui::Context) {
        if let Some(tray) = self.tray.as_mut() {
            tray.sync(ctx, self.lang);
        }

        for action in tray::take_actions() {
            match action {
                tray::TrayAction::Open => {}
                tray::TrayAction::Toggle => {
                    if self.watcher_running() {
                        self.stop_watcher();
                    } else {
                        self.start_watcher();
                    }
                }
                tray::TrayAction::Restart => self.restart_watcher(),
                tray::TrayAction::Organize => {
                    self.tab = Tab::Vista;
                    if self.plan.is_some() {
                        self.ask_organize();
                    } else {
                        self.tray_organize = true;
                        self.scan();
                    }
                }
                tray::TrayAction::Quit => {
                    self.quit_requested = true;
                    self.stop_watcher();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn sync_window_state(&mut self, ctx: &egui::Context) {
        let (inner, outer) = ctx.input(|input| {
            let viewport = input.viewport();
            (viewport.inner_rect, viewport.outer_rect)
        });
        if let Some(rect) = inner {
            if rect.width() > 100.0 && rect.height() > 100.0 {
                self.settings.window_size = (rect.width(), rect.height());
            }
        }
        if let Some(rect) = outer {
            if rect.min.x > -10_000.0 && rect.min.y > -10_000.0 {
                self.settings.window_pos = (rect.min.x, rect.min.y);
            }
        }
    }

    #[cfg(windows)]
    fn handle_close(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return;
        }
        self.sync_window_state(ctx);
        self.save();
        if self.quit_requested {
            return;
        }
        if self.watcher_running() && self.tray.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Some(reply) = self.worker.poll() {
            match reply {
                Reply::Plan(result) => {
                    self.scanning = false;
                    match result {
                        Ok(plan) => {
                            self.plan = Some(plan);
                            if self.tray_organize {
                                self.tray_organize = false;
                                self.ask_organize();
                            }
                        }
                        Err(err) => {
                            self.plan = None;
                            self.tray_organize = false;
                            self.notice = Some(Notice::error(err));
                        }
                    }
                }
                Reply::Organized(result) => {
                    self.organizing = false;
                    match result {
                        Ok(summary) => {
                            let total = summary.moved + summary.copied + summary.trashed;
                            if total > 0 {
                                if !self.settings.silent {
                                    self.notice = Some(Notice::info(i18n::organized_notice(
                                        self.lang,
                                        summary.moved,
                                        summary.copied,
                                        summary.trashed,
                                    )));
                                }
                            } else {
                                self.notice = Some(Notice::info(
                                    i18n::t(self.lang, Key::EmptyNoPlan).to_string(),
                                ));
                            }

                            if let Some((_, reason)) = summary.failures.first() {
                                self.notice = Some(Notice::error(reason.clone()));
                            }

                            self.plan = None;
                            self.scan();
                            self.refresh_history();
                        }
                        Err(err) => self.notice = Some(Notice::error(err)),
                    }
                }
                Reply::Undone(result) => {
                    self.organizing = false;
                    match result {
                        Ok(report) => {
                            let text = if report.restored == 0 && report.skipped == 0 {
                                i18n::t(self.lang, Key::UndoNothing).to_string()
                            } else if report.skipped == 0 {
                                i18n::undo_done(self.lang, report.restored)
                            } else {
                                i18n::undo_partial(self.lang, report.restored, report.skipped)
                            };
                            if !self.settings.silent {
                                self.notice = Some(Notice::info(text));
                            }
                            self.scan();
                            self.refresh_history();
                        }
                        Err(err) => self.notice = Some(Notice::error(err)),
                    }
                }
                Reply::History(history, totals) => {
                    self.history = history;
                    self.totals = totals;
                }
            }
        }

        // Si el proceso de vigilancia murio, se refleja en la barra de estado y
        // se reintenta solo cuando la salida no fue pedida por esta ventana.
        self.supervise_watcher(ctx);
        self.retry_watcher(ctx);

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.confirm = None;
        }

        if !ctx.egui_wants_keyboard_input() {
            let undo =
                ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::Z));
            if undo {
                self.ask_undo_last();
            }

            let rescan = ctx.input(|input| input.key_pressed(egui::Key::F5));
            if rescan {
                self.scan();
            }

            let open =
                ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::O));
            if open {
                self.pick_folder();
            }
        }
    }

    pub fn pick_folder(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, Key::ChooseFolder))
            .pick_folder()
        else {
            return;
        };

        self.add_root(&path);
    }

    pub fn watcher_running(&mut self) -> bool {
        let Some(child) = self.watch_child.as_mut() else {
            return self.external_watch;
        };
        // No se limpia aqui: `poll` es quien distingue una salida normal de un
        // fallo y decide si toca reintentar.
        matches!(child.try_wait(), Ok(None))
    }

    pub fn start_watcher(&mut self) {
        if self.watcher_running() {
            return;
        }

        self.watch_attempts.clear();
        self.watch_retry_at = None;
        self.watch_failed = false;
        self.watch_supervisor = true;

        if let Err(err) = self.spawn_watcher() {
            self.watch_supervisor = false;
            self.notice = Some(Notice::error(err.describe(self.lang)));
            crate::logging::log_message(&err.describe(self.lang));
        }
    }

    /// Arranca el hijo oculto. Devuelve error solo si el sistema no deja crear
    /// el proceso; la deteccion de un vigilante externo no es un error.
    fn spawn_watcher(&mut self) -> Result<(), Error> {
        // Si el vigilante lo arranco el inicio de Windows, no es hijo de esta
        // ventana. Se detecta con el mismo mutex y se controla con watch.stop.
        #[cfg(windows)]
        if let Ok(probe) = single_instance::SingleInstance::new(&config::instance_name("watch")) {
            if !probe.is_single() {
                self.external_watch = true;
                self.watch_supervisor = false;
                self.settings.background = true;
                self.save();
                return Ok(());
            }
        }

        let exe = std::env::current_exe()
            .map_err(|err| Error::io(Path::new("smart-organizer.exe"), err))?;

        // El mismo ejecutable hace de vigilante oculto. Asi no depende de que
        // el CLI este al lado y solo puede quedar un proceso por configuracion.
        let _ = config::clear_watch_stop();
        let mut command = Process::new(&exe);
        command
            .arg("--watch")
            .arg("--hidden")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        match command.spawn() {
            Ok(child) => {
                self.watch_child = Some(child);
                self.external_watch = false;
                self.settings.background = true;
                self.save();
                Ok(())
            }
            Err(err) => Err(Error::io(&exe, err)),
        }
    }

    pub fn stop_watcher(&mut self) {
        // Se marca como parada intencional antes de matar: el sondeo no debe
        // confundirla con un fallo ni reprogramar un reintento.
        self.watch_supervisor = false;
        self.watch_retry_at = None;
        self.watch_attempts.clear();
        self.watch_failed = false;

        if let Some(mut child) = self.watch_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        // Tambien detiene al vigilante que arranco el inicio de Windows, que no
        // es hijo de esta ventana.
        let _ = config::request_watch_stop();
        self.external_watch = false;
        self.settings.background = false;
        self.save();
    }

    /// Reinicio pedido por la persona. Si el vigilante lo arranco el inicio de
    /// Windows, se espera a que suelte el mutex antes de crear otro.
    pub fn restart_watcher(&mut self) {
        self.stop_watcher();

        #[cfg(windows)]
        {
            let name = config::instance_name("watch");
            for _ in 0..40 {
                match single_instance::SingleInstance::new(&name) {
                    Ok(probe) if probe.is_single() => break,
                    _ => thread::sleep(Duration::from_millis(50)),
                }
            }
        }

        self.start_watcher();
    }

    fn supervise_watcher(&mut self, ctx: &egui::Context) {
        let Some(child) = self.watch_child.as_mut() else {
            return;
        };
        if let Ok(None) = child.try_wait() {
            return;
        }

        self.watch_child = None;
        self.external_watch = false;
        if !self.watch_supervisor || !self.settings.background || self.watch_failed {
            return;
        }

        let now = Instant::now();
        let window = Duration::from_secs(600);
        self.watch_attempts
            .retain(|at| now.duration_since(*at) < window);

        if self.watch_attempts.len() >= 5 {
            self.watch_failed = true;
            self.watch_retry_at = None;
            let text = i18n::t(self.lang, Key::WatchFailed).to_string();
            crate::logging::log_message(&text);
            self.notice = Some(Notice::error(text));
            return;
        }

        let delay = Duration::from_secs(1u64 << self.watch_attempts.len().min(4));
        self.watch_retry_at = Some(now + delay);
        ctx.request_repaint_after(delay);
    }

    fn retry_watcher(&mut self, ctx: &egui::Context) {
        let Some(at) = self.watch_retry_at else {
            return;
        };
        let now = Instant::now();
        if now < at {
            ctx.request_repaint_after(at - now);
            return;
        }

        self.watch_retry_at = None;
        self.watch_attempts.push(now);

        if let Err(err) = self.spawn_watcher() {
            crate::logging::log_message(&err.describe(self.lang));
            // El fallo al crear el proceso cuenta como intento y vuelve a la
            // misma espera, hasta el limite de cinco.
            if self.watch_attempts.len() >= 5 {
                self.watch_failed = true;
                let text = i18n::t(self.lang, Key::WatchFailed).to_string();
                self.notice = Some(Notice::error(text));
            } else {
                let delay = Duration::from_secs(1u64 << self.watch_attempts.len().min(4));
                self.watch_retry_at = Some(now + delay);
                ctx.request_repaint_after(delay);
            }
        }
    }

    pub fn open_path(&mut self, path: &std::path::Path) {
        let result = {
            #[cfg(windows)]
            {
                Process::new("explorer").arg(path).spawn()
            }
            #[cfg(target_os = "macos")]
            {
                Process::new("open").arg(path).spawn()
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                Process::new("xdg-open").arg(path).spawn()
            }
        };

        match result {
            Ok(_) => {}
            Err(err) => {
                self.notice = Some(Notice::error(Error::io(path, err).describe(self.lang)));
            }
        }
    }

    pub fn export_csv(&mut self) {
        if self.history.is_empty() {
            self.notice = Some(Notice::info(
                i18n::t(self.lang, Key::ExportEmpty).to_string(),
            ));
            return;
        }

        let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, Key::ExportCsv))
            .set_file_name("historial-smart-organizer.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };

        match organize::export_csv(&path, self.lang) {
            Ok(rows) if rows > 0 => {
                self.notice = Some(Notice::info(i18n::export_done(self.lang, &path)));
            }
            Ok(_) => {
                self.notice = Some(Notice::info(
                    i18n::t(self.lang, Key::ExportEmpty).to_string(),
                ));
            }
            Err(err) => self.notice = Some(Notice::error(err.describe(self.lang))),
        }
    }

    fn apply_confirm(&mut self, confirm: Confirm, ctx: &egui::Context) {
        match confirm {
            Confirm::Move { .. } => self.organize_now(),
            Confirm::Undo { batch, .. } => {
                self.organizing = true;
                self.worker.send(Command::Undo {
                    batch,
                    lang: self.lang,
                });
            }
            Confirm::Reset => {
                self.stop_watcher();
                self.settings = Settings::default();
                self.lang = Lang::default();
                self.accent_draft = self.settings.accent.clone();
                self.temp_text = self.settings.temp_extensions.join(", ");
                self.skip_text = self.settings.skip_dirs.join(", ");
                self.excludes = String::new();
                self.file_excludes = String::new();
                self.root_exclude_files.clear();
                self.root_exclude_dirs.clear();
                self.scope = Scope::Global;
                self.rule_draft = RuleDraft::default();
                self.folder_edit = None;
                self.save();
                self.plan = None;
                self.history.clear();
                self.totals = Totals::default();
                self.refresh_history();
                self.notice = Some(Notice::info(i18n::t(self.lang, Key::ResetDone).to_string()));
            }
            Confirm::FolderRename {
                scope, old, new, ..
            } => {
                let mut moved = 0usize;
                let mut failure = None;
                for root in self.scope_roots(scope) {
                    let from = root.join(&old);
                    let to = root.join(&new);
                    match mover::move_folder(&from, &to) {
                        Ok(count) => moved += count,
                        Err(err) => {
                            failure = Some(err.describe(self.lang));
                            break;
                        }
                    }
                }

                if let Some(message) = failure {
                    self.notice = Some(Notice::error(message));
                } else if moved > 0 {
                    self.notice = Some(Notice::info(i18n::folder_moved(self.lang, moved)));
                } else {
                    self.notice = Some(Notice::info(
                        i18n::t(self.lang, Key::FolderMoved).to_string(),
                    ));
                }
                self.scan();
            }
            Confirm::Duplicates { .. } => self.trash_duplicates(ctx),
            Confirm::Import { settings } => {
                self.stop_watcher();
                self.settings = *settings;
                self.settings.normalize();
                self.lang = if self.settings.language == "en" {
                    Lang::En
                } else {
                    Lang::Es
                };
                self.accent_draft = self.settings.accent.clone();
                self.temp_text = self.settings.temp_extensions.join(", ");
                self.skip_text = self.settings.skip_dirs.join(", ");
                self.excludes = self.settings.watch.exclude_dirs.join(", ");
                self.file_excludes = self.settings.watch.exclude_files.join(", ");
                self.root_exclude_files = self
                    .settings
                    .roots
                    .iter()
                    .map(|root| root.overrides.exclude_files.join(", "))
                    .collect();
                self.root_exclude_dirs = self
                    .settings
                    .roots
                    .iter()
                    .map(|root| root.overrides.exclude_dirs.join(", "))
                    .collect();
                self.scope = Scope::Global;
                self.rule_draft = RuleDraft::default();
                self.plan = None;
                if !self.settings.advanced && self.tab == Tab::Organizar {
                    self.tab = Tab::Ajustes;
                }
                self.save();
                self.scan();
                self.notice = Some(Notice::info(
                    i18n::t(self.lang, Key::ImportDone).to_string(),
                ));
            }
        }
    }

    fn render_confirm(&mut self, ctx: &egui::Context) {
        let Some(confirm) = self.confirm.take() else {
            return;
        };

        let palette = theme::Palette::for_mode(
            ctx.style_of(ctx.theme()).visuals.dark_mode,
            &self.settings.accent,
        );

        let (title, body, confirm_label, cancel_label) = match &confirm {
            Confirm::Move { files, trash } => {
                let mut body = i18n::confirm_move_body(self.lang, *files);
                if *trash > 0 {
                    body.push('\n');
                    body.push('\n');
                    body.push_str(i18n::t(self.lang, Key::TrashNote));
                }
                (
                    i18n::t(self.lang, Key::ConfirmMoveTitle),
                    body,
                    i18n::t(self.lang, Key::ConfirmMoveYes),
                    i18n::t(self.lang, Key::Cancel),
                )
            }
            Confirm::Undo { files, .. } => (
                i18n::t(self.lang, Key::UndoConfirmTitle),
                i18n::undo_confirm_body(self.lang, *files),
                i18n::t(self.lang, Key::Undo),
                i18n::t(self.lang, Key::Cancel),
            ),
            Confirm::Reset => (
                i18n::t(self.lang, Key::ResetTitle),
                i18n::t(self.lang, Key::ResetBody).to_string(),
                i18n::t(self.lang, Key::ResetSettings),
                i18n::t(self.lang, Key::Cancel),
            ),
            Confirm::FolderRename {
                old, new, entries, ..
            } => (
                i18n::t(self.lang, Key::FolderRenameTitle),
                {
                    let mut body = i18n::folder_rename_body(self.lang, old, new);
                    if *entries > 0 {
                        body.push_str(&format!(
                            "\n{}: {entries}",
                            i18n::t(self.lang, Key::ItemsLabel)
                        ));
                    }
                    body
                },
                i18n::t(self.lang, Key::FolderMoveContent),
                i18n::t(self.lang, Key::FolderLeaveContent),
            ),
            Confirm::Duplicates { files, bytes } => (
                i18n::t(self.lang, Key::DuplicatesTitle),
                i18n::duplicates_confirm(self.lang, *files, *bytes),
                i18n::t(self.lang, Key::DuplicatesAction),
                i18n::t(self.lang, Key::Cancel),
            ),
            Confirm::Import { settings } => (
                i18n::t(self.lang, Key::ImportPreviewTitle),
                format!(
                    "{} {}",
                    i18n::import_preview(
                        self.lang,
                        settings.roots.len(),
                        settings.categories.len(),
                        settings.rules.len()
                    ),
                    i18n::t(self.lang, Key::ImportPreviewBody)
                ),
                i18n::t(self.lang, Key::ImportConfig),
                i18n::t(self.lang, Key::Cancel),
            ),
        };

        let answer =
            widgets::modal_confirm(ctx, &palette, title, &body, confirm_label, cancel_label);

        match answer {
            Some(true) => self.apply_confirm(confirm, ctx),
            Some(false) => {
                if let Confirm::FolderRename { scope, old, .. } = confirm {
                    self.leave_folder_content(scope, &old);
                }
            }
            None => self.confirm = Some(confirm),
        }
    }

    /// El contenido se queda en la carpeta vieja. Esa carpeta deja de ser un
    /// destino conocido, asi que se excluye para que el siguiente escaneo no la
    /// vuelva a mover sin permiso.
    fn leave_folder_content(&mut self, scope: Scope, old: &str) {
        let pattern = globset::escape(old);
        let roots = match scope {
            Scope::Global => self.settings.roots.iter_mut().collect::<Vec<_>>(),
            Scope::Root(index) => self.settings.roots.get_mut(index).into_iter().collect(),
        };
        for root in roots {
            if !root
                .overrides
                .exclude_dirs
                .iter()
                .any(|value| value == &pattern)
            {
                root.overrides.exclude_dirs.push(pattern.clone());
            }
        }
        self.root_exclude_dirs = self
            .settings
            .roots
            .iter()
            .map(|root| root.overrides.exclude_dirs.join(", "))
            .collect();
        self.save();
        self.scan();
        self.notice = Some(Notice::info(i18n::folder_left_excluded(self.lang, old)));
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let palette = theme::palette(ui, &self.settings);

        egui::Frame::NONE
            .fill(palette.bg)
            .inner_margin(Margin {
                left: 20,
                right: 20,
                top: 12,
                bottom: 10,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(i18n::t(self.lang, Key::AppName))
                            .size(15.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .size(13.0)
                            .color(palette.muted),
                    );

                    ui.add_space(16.0);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let compact = ui.available_width() < 620.0;
                        let organize_enabled = !self.organizing
                            && self.plan.as_ref().is_some_and(|plan| !plan.is_empty());
                        let undo_enabled = self
                            .history
                            .iter()
                            .any(|batch| !batch.undone && batch.files > 0);

                        if compact {
                            if widgets::icon_action(
                                ui,
                                &palette,
                                widgets::ActionIcon::Organize,
                                organize_enabled,
                                i18n::t(self.lang, Key::OrganizeNow),
                            )
                            .clicked()
                                && organize_enabled
                            {
                                self.ask_organize();
                            }
                            if widgets::icon_action(
                                ui,
                                &palette,
                                widgets::ActionIcon::Undo,
                                undo_enabled,
                                i18n::t(self.lang, Key::Undo),
                            )
                            .clicked()
                                && undo_enabled
                            {
                                self.ask_undo_last();
                            }
                        } else {
                            let organize = widgets::accent_button_enabled(
                                ui,
                                &palette,
                                i18n::t(self.lang, Key::OrganizeNow),
                                organize_enabled,
                            );
                            if organize.clicked() {
                                self.ask_organize();
                            }
                            if !organize_enabled {
                                organize.on_disabled_hover_text(i18n::t(
                                    self.lang,
                                    Key::OrganizeDisabledHint,
                                ));
                            }

                            let undo = ui.add_enabled_ui(undo_enabled, |ui| {
                                widgets::ghost_button(ui, &palette, i18n::t(self.lang, Key::Undo))
                            });
                            if undo.inner.clicked() {
                                self.ask_undo_last();
                            }
                            if !undo_enabled {
                                undo.inner.on_disabled_hover_text(i18n::t(
                                    self.lang,
                                    Key::UndoDisabledHint,
                                ));
                            }
                        }

                        if !compact && let Some(root) = self.settings.roots.first() {
                            let name = root
                                .path
                                .file_name()
                                .map(|name| name.to_string_lossy().into_owned())
                                .unwrap_or_else(|| root.path.display().to_string());
                            widgets::folder_chip(ui, &palette, &name);
                        }
                    });
                });
            });

        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::ZERO, palette.hairline());
    }

    fn nav_bar(&mut self, ui: &mut egui::Ui) {
        let palette = theme::palette(ui, &self.settings);
        egui::Frame::NONE
            .fill(palette.bg)
            .inner_margin(Margin {
                left: 12,
                right: 12,
                top: 6,
                bottom: 8,
            })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for tab in self.visible_tabs() {
                        if widgets::nav_entry(ui, &palette, tab.label(self.lang), self.tab == tab)
                            .clicked()
                        {
                            self.tab = tab;
                        }
                        ui.add_space(2.0);
                    }
                });
            });
    }

    pub fn visible_tabs(&self) -> Vec<Tab> {
        let mut tabs = vec![Tab::Inicio, Tab::Vista, Tab::Historial, Tab::Ajustes];
        if self.settings.advanced {
            tabs.insert(3, Tab::Organizar);
        }
        tabs
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let palette = theme::palette(ui, &self.settings);
        let running = self.watcher_running();
        let folders = self.settings.active_roots().len();

        egui::Frame::NONE
            .fill(palette.surface)
            .inner_margin(Margin::symmetric(20, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot, _) =
                        ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
                    let (dot_color, text, color) = if running {
                        (
                            palette.success,
                            i18n::t(self.lang, Key::WatchOn),
                            palette.success,
                        )
                    } else if self.watch_failed {
                        (
                            palette.error,
                            i18n::t(self.lang, Key::WatchFailedShort),
                            palette.error,
                        )
                    } else if self.watch_retry_at.is_some() {
                        (
                            palette.accent,
                            i18n::t(self.lang, Key::WatchRetrying),
                            palette.accent,
                        )
                    } else {
                        (
                            palette.muted,
                            i18n::t(self.lang, Key::WatchOff),
                            palette.muted,
                        )
                    };
                    ui.painter().circle_filled(dot.center(), 3.5, dot_color);
                    let status = ui.add(
                        egui::Button::new(RichText::new(text).size(13.0).color(color))
                            .fill(Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(4))
                            .min_size(egui::vec2(0.0, 28.0)),
                    );
                    if status.clicked() {
                        if running {
                            self.stop_watcher();
                        } else {
                            self.start_watcher();
                        }
                    }
                    let hint = if self.watch_failed {
                        Key::WatchFailed
                    } else {
                        Key::WatchHint
                    };
                    status.on_hover_text(i18n::t(self.lang, hint));
                    if (running || self.watch_failed || self.watch_retry_at.is_some())
                        && widgets::ghost_button(
                            ui,
                            &palette,
                            i18n::t(self.lang, Key::RestartWatch),
                        )
                        .clicked()
                    {
                        self.restart_watcher();
                    }
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(i18n::active_folders(self.lang, folders))
                            .size(13.0)
                            .color(palette.muted),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(at) = self.save_shown_at {
                            if at.elapsed() < Duration::from_secs(2) {
                                ui.label(
                                    RichText::new(i18n::t(self.lang, Key::Saved))
                                        .size(13.0)
                                        .color(palette.success),
                                );
                                ui.ctx().request_repaint_after(Duration::from_millis(250));
                            } else {
                                self.save_shown_at = None;
                            }
                        }
                        ui.label(
                            RichText::new(i18n::t(self.lang, Key::ClipboardInfo))
                                .size(13.0)
                                .color(palette.text),
                        );
                    });
                });
            });
    }

    fn notice_bar(&mut self, ui: &mut egui::Ui) {
        let Some(notice) = &self.notice else {
            return;
        };

        let palette = theme::palette(ui, &self.settings);
        let color = if notice.error {
            palette.error
        } else {
            palette.success
        };
        let text = notice.text.clone();

        egui::Frame::NONE
            .fill(palette.tint(color))
            .stroke(egui::Stroke::new(1.0, color))
            .corner_radius(egui::CornerRadius::same(4))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(text).size(13.0).color(palette.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(i18n::t(self.lang, Key::Dismiss))
                                        .size(13.0)
                                        .color(palette.muted),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(0.0, 28.0)),
                            )
                            .clicked()
                        {
                            self.notice = None;
                        }
                        if self.undo_change.is_some()
                            && widgets::link_button(ui, &palette, i18n::t(self.lang, Key::Undo))
                                .clicked()
                        {
                            self.undo_pending_change();
                        }
                    });
                });
            });

        ui.add_space(10.0);
    }

    fn drop_overlay(&self, ctx: &egui::Context) {
        let hovering = ctx.input(|input| !input.raw.hovered_files.is_empty());
        if !hovering {
            return;
        }

        let palette = theme::Palette::for_mode(
            ctx.style_of(ctx.theme()).visuals.dark_mode,
            &self.settings.accent,
        );
        let rect = ctx.content_rect();
        #[allow(deprecated)]
        let overlay_id = egui::Id::new("drop-overlay");
        egui::Area::new(overlay_id)
            .order(egui::Order::Foreground)
            .fixed_pos(rect.min)
            .interactable(false)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::ZERO,
                    Color32::from_black_alpha(96),
                );
                let card = egui::Rect::from_center_size(
                    rect.center(),
                    egui::vec2(rect.width().min(380.0), 62.0),
                );
                let painter = ui.painter();
                painter.rect_filled(card, egui::CornerRadius::same(4), palette.surface);
                painter.rect_stroke(
                    card,
                    egui::CornerRadius::same(4),
                    egui::Stroke::new(2.0, palette.accent),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    card.center(),
                    egui::Align2::CENTER_CENTER,
                    i18n::t(self.lang, Key::DropHere),
                    egui::FontId::proportional(15.0),
                    palette.text,
                );
            });
    }

    fn render_onboarding(&mut self, ctx: &egui::Context) {
        if !self.settings.roots.is_empty() || self.settings.onboarded {
            return;
        }

        let palette = theme::Palette::for_mode(
            ctx.style_of(ctx.theme()).visuals.dark_mode,
            &self.settings.accent,
        );
        let lang = self.lang;
        let shortcuts = self.available_shortcuts();
        let has_roots = !self.settings.roots.is_empty();
        let mut close = false;
        let mut finish = false;
        let mut preview = false;

        #[allow(deprecated)]
        let id = egui::Id::new("onboarding-modal");
        egui::Modal::new(id).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.label(
                RichText::new(i18n::t(lang, Key::OnboardingTitle))
                    .size(20.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(i18n::onboarding_step(lang, self.onboard_step + 1))
                    .size(13.0)
                    .color(palette.muted),
            );
            ui.add_space(8.0);

            let (track, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(track, egui::CornerRadius::same(2), palette.surface_alt);
            let segment = track.width() / 3.0;
            for index in 0..=self.onboard_step {
                let part = egui::Rect::from_min_size(
                    track.min + egui::vec2(segment * index as f32, 0.0),
                    egui::vec2((segment - 4.0).max(1.0), track.height()),
                );
                ui.painter()
                    .rect_filled(part, egui::CornerRadius::same(2), palette.accent);
            }

            ui.add_space(16.0);

            match self.onboard_step {
                0 => {
                    ui.label(
                        RichText::new(i18n::t(lang, Key::OnboardingStep1Title))
                            .size(16.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.add_space(4.0);
                    widgets::small_muted(ui, &palette, i18n::t(lang, Key::OnboardingStep1Body));
                    ui.add_space(10.0);
                    if has_roots {
                        ui.label(
                            RichText::new(i18n::t(lang, Key::OnboardingFolderAdded))
                                .size(13.0)
                                .color(palette.success),
                        );
                    }
                    if !shortcuts.is_empty() {
                        widgets::small_muted(ui, &palette, i18n::t(lang, Key::ShortcutsHint));
                        ui.horizontal_wrapped(|ui| {
                            for (key, path) in &shortcuts {
                                if widgets::ghost_button(
                                    ui,
                                    &palette,
                                    &format!("+ {}", i18n::t(lang, *key)),
                                )
                                .clicked()
                                {
                                    self.add_root(path);
                                }
                            }
                        });
                        ui.add_space(8.0);
                    }
                    if widgets::accent_button(ui, &palette, i18n::t(lang, Key::OnboardingPick))
                        .clicked()
                    {
                        self.pick_folder();
                    }
                    if !has_roots {
                        ui.add_space(6.0);
                        widgets::small_muted(ui, &palette, i18n::t(lang, Key::OnboardingNoFolders));
                    }
                }
                1 => {
                    ui.label(
                        RichText::new(i18n::t(lang, Key::OnboardingStep2Title))
                            .size(16.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.add_space(4.0);
                    widgets::small_muted(ui, &palette, i18n::t(lang, Key::OnboardingStep2Body));
                    ui.add_space(10.0);
                    let mut by_date = self.settings.subfolders.by_year_month;
                    if ui
                        .radio(!by_date, i18n::t(lang, Key::StyleByType))
                        .clicked()
                    {
                        by_date = false;
                    }
                    if ui
                        .radio(by_date, i18n::t(lang, Key::StyleByTypeAndDate))
                        .clicked()
                    {
                        by_date = true;
                    }
                    self.settings.subfolders.by_type = true;
                    self.settings.subfolders.by_year_month = by_date;
                }
                _ => {
                    ui.label(
                        RichText::new(i18n::t(lang, Key::OnboardingStep3Title))
                            .size(16.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.add_space(4.0);
                    widgets::small_muted(ui, &palette, i18n::t(lang, Key::OnboardingStep3Body));
                    ui.add_space(10.0);
                    match self.plan.as_ref().map(|plan| plan.selected_count()) {
                        Some(count) if count > 0 => {
                            widgets::pill(
                                ui,
                                &palette,
                                &i18n::files_label(lang, count),
                                palette.accent_text,
                            );
                        }
                        _ => {
                            widgets::small_muted(ui, &palette, i18n::t(lang, Key::EmptyNoPlan));
                        }
                    }
                }
            }

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if widgets::link_button(ui, &palette, i18n::t(lang, Key::OnboardingSkip)).clicked()
                {
                    close = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.onboard_step == 0 {
                        let enabled = has_roots;
                        let next = ui.add_enabled(
                            enabled,
                            egui::Button::new(
                                RichText::new(i18n::t(lang, Key::WizardNext))
                                    .color(if enabled {
                                        palette.on_accent
                                    } else {
                                        palette.muted
                                    })
                                    .strong(),
                            )
                            .fill(if enabled {
                                palette.accent
                            } else {
                                palette.surface_alt
                            })
                            .corner_radius(egui::CornerRadius::same(6))
                            .min_size(egui::vec2(0.0, 36.0)),
                        );
                        if next.clicked() {
                            self.onboard_step = 1;
                        }
                    } else if self.onboard_step == 1 {
                        if widgets::ghost_button(ui, &palette, i18n::t(lang, Key::WizardBack))
                            .clicked()
                        {
                            self.onboard_step = 0;
                        }
                        if widgets::accent_button(ui, &palette, i18n::t(lang, Key::WizardNext))
                            .clicked()
                        {
                            self.onboard_step = 2;
                            self.scan();
                        }
                    } else {
                        if widgets::ghost_button(ui, &palette, i18n::t(lang, Key::WizardBack))
                            .clicked()
                        {
                            self.onboard_step = 1;
                        }
                        if widgets::accent_button(ui, &palette, i18n::t(lang, Key::OrganizeNow))
                            .clicked()
                        {
                            finish = true;
                        }
                        if widgets::ghost_button(
                            ui,
                            &palette,
                            i18n::t(lang, Key::OnboardingPreview),
                        )
                        .clicked()
                        {
                            preview = true;
                        }
                    }
                });
            });
        });

        if close || finish || preview {
            self.settings.onboarded = true;
            self.save();
        }
        if preview {
            if self.plan.is_none() {
                self.scan();
            }
            self.tab = Tab::Vista;
        }
        if finish {
            self.tab = Tab::Vista;
            if self.plan.is_none() {
                self.scan();
            } else {
                self.ask_organize();
            }
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(windows)]
        self.handle_close(ctx);
        #[cfg(not(windows))]
        if ctx.input(|input| input.viewport().close_requested()) {
            self.sync_window_state(ctx);
            self.save();
        }
        self.poll(ctx);
        self.poll_duplicates();
        #[cfg(windows)]
        self.poll_tray(ctx);
        self.sync_window_state(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Minimizada no hay nada que pintar. El sistema puede pedir un cuadro
        // al mover o restaurar; el trabajo de fondo ya corre en `logic`.
        if ui
            .ctx()
            .input(|input| input.viewport().minimized.unwrap_or(false))
        {
            return;
        }

        if !self.settings.advanced {
            if self.tab == Tab::Organizar {
                self.tab = Tab::Ajustes;
            }
            if matches!(
                self.section,
                SettingsSection::Reglas
                    | SettingsSection::Vigilancia
                    | SettingsSection::Duplicados
                    | SettingsSection::Avanzado
            ) {
                self.section = SettingsSection::General;
            }
        }

        let dropped: Vec<PathBuf> = ui.ctx().input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        if !dropped.is_empty() {
            self.add_dropped(&dropped);
        }

        let key = (
            self.settings.theme,
            self.settings.accent.clone(),
            self.settings.text_size,
            self.settings.density,
            ui.ctx().theme() == egui::Theme::Dark,
        );
        if self.applied.as_ref() != Some(&key) {
            theme::apply(ui.ctx(), &self.settings);
            self.applied = Some(key);
        }

        egui::Panel::top("barra")
            .frame(egui::Frame::NONE.fill(theme::palette(ui, &self.settings).bg))
            .show(ui, |ui| self.top_bar(ui));

        egui::Panel::left("navegacion")
            .exact_size(172.0)
            .frame(egui::Frame::NONE.fill(theme::palette(ui, &self.settings).bg))
            .show(ui, |ui| self.nav_bar(ui));

        egui::Panel::bottom("estado")
            .frame(egui::Frame::NONE.fill(theme::palette(ui, &self.settings).surface))
            .show(ui, |ui| self.status_bar(ui));

        let palette = theme::palette(ui, &self.settings);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(palette.bg).inner_margin(Margin {
                left: 20,
                right: 20,
                top: 14,
                bottom: 14,
            }))
            .show(ui, |ui| {
                self.notice_bar(ui);

                match self.tab {
                    Tab::Inicio => inicio::show(self, ui),
                    Tab::Vista => vista_previa::show(self, ui),
                    Tab::Historial => historial::show(self, ui),
                    Tab::Ajustes => ajustes::show(self, ui),
                    Tab::Organizar => organizar::show(self, ui),
                }
            });

        if self.confirm.is_some() {
            self.render_confirm(ui.ctx());
        }
        self.render_onboarding(ui.ctx());
        self.drop_overlay(ui.ctx());
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.settings.normalize();
        let _ = self.settings.save();
        if let Some(mut child) = self.watch_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if self.external_watch {
            let _ = config::request_watch_stop();
        }
    }
}

/// Punto de entrada que usa `main.rs`.
pub fn run(lang: Lang) -> Result<(), Error> {
    let settings = Settings::load().unwrap_or_default();
    let size = settings.window_size;
    let position = settings.window_pos;
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([size.0, size.1])
        .with_min_inner_size([900.0, 560.0])
        .with_icon(icon::viewport_icon())
        .with_title(i18n::t(lang, Key::AppName));
    if position.0 >= 0.0 && position.1 >= 0.0 {
        viewport = viewport.with_position([position.0, position.1]);
    }

    let options = eframe::NativeOptions {
        centered: position.0 < 0.0 || position.1 < 0.0,
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        i18n::t(lang, Key::AppName),
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .map_err(|err| {
        let detail = err.to_string();
        let lower = detail.to_lowercase();
        if ["opengl", "glow", "wgl", "egl", "glx", "renderer"]
            .iter()
            .any(|needle| lower.contains(needle))
        {
            Error::GuiOpenGl { detail }
        } else {
            Error::Gui { detail }
        }
    })
}

/// Texto de la ultima organizacion, en la zona local.
pub fn last_run_label(app: &App) -> String {
    match app.history.first() {
        Some(batch) => i18n::last_organized(app.lang, &crate::journal::format_stamp(&batch.at)),
        None => i18n::t(app.lang, Key::LastOrganizeNever).to_string(),
    }
}

/// Ayuda para abrir la carpeta de datos del programa.
pub fn data_folder() -> Result<std::path::PathBuf, Error> {
    let dir = config::config_dir()?;
    std::fs::create_dir_all(&dir).map_err(|err| Error::io(&dir, err))?;
    Ok(dir)
}
