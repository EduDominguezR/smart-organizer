//! Pantalla de duplicados: busqueda bajo demanda y envio seguro a la papelera.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Layout, RichText};

use super::theme::{self, Palette};
use super::widgets;
use super::{App, Notice};
use crate::duplicates::{self, DuplicateGroup, TrashOutcome};
use crate::i18n::{self, Key};

enum Message {
    Progress(usize, usize),
    Found(Result<Option<Vec<DuplicateGroup>>, String>),
    Trashed(Result<TrashOutcome, String>),
}

#[derive(Default)]
pub struct State {
    pub groups: Vec<DuplicateGroup>,
    pub selected: HashSet<PathBuf>,
    pub scanning: bool,
    pub trashing: bool,
    pub done: bool,
    pub progress: (usize, usize),
    pub cancel: Option<Arc<AtomicBool>>,
    receiver: Option<Receiver<Message>>,
}

impl State {
    pub fn selected_bytes(&self) -> u64 {
        self.groups
            .iter()
            .map(|group| {
                let count = group
                    .files
                    .iter()
                    .filter(|file| self.selected.contains(&file.path))
                    .count();
                group.size.saturating_mul(count as u64)
            })
            .sum()
    }

    pub fn clear(&mut self) {
        self.groups.clear();
        self.selected.clear();
        self.done = false;
        self.progress = (0, 0);
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.receiver = None;
    }
}

impl App {
    pub fn start_duplicates(&mut self, ctx: &egui::Context) {
        if self.duplicates.scanning || self.duplicates.trashing {
            return;
        }
        self.duplicates.clear();
        self.duplicates.scanning = true;
        self.tab = super::Tab::Ajustes;
        self.section = super::SettingsSection::Duplicados;
        self.settings.advanced = true;
        self.save();

        let settings = self.settings.clone();
        let lang = self.lang;
        let cancel = Arc::new(AtomicBool::new(false));
        self.duplicates.cancel = Some(cancel.clone());
        let (sender, receiver) = mpsc::channel();
        self.duplicates.receiver = Some(receiver);
        let repaint = ctx.clone();

        thread::spawn(move || {
            crate::priority::lower_thread();
            let mut last = Instant::now();
            let result = duplicates::find(&settings, &cancel, |done, total| {
                if done % 32 == 0 || done == total || last.elapsed() > Duration::from_millis(120) {
                    last = Instant::now();
                    let _ = sender.send(Message::Progress(done, total));
                    repaint.request_repaint();
                }
            });
            let _ = sender.send(Message::Found(result.map_err(|err| err.describe(lang))));
            repaint.request_repaint();
        });
    }

    pub fn cancel_duplicates(&mut self) {
        if let Some(cancel) = self.duplicates.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    pub fn poll_duplicates(&mut self) {
        let Some(receiver) = self.duplicates.receiver.take() else {
            return;
        };
        let mut keep = true;
        while let Ok(message) = receiver.try_recv() {
            match message {
                Message::Progress(done, total) => self.duplicates.progress = (done, total),
                Message::Found(result) => {
                    keep = false;
                    self.duplicates.scanning = false;
                    self.duplicates.cancel = None;
                    match result {
                        Ok(Some(groups)) => {
                            self.duplicates.selected.clear();
                            for group in &groups {
                                for (index, file) in group.files.iter().enumerate() {
                                    if index != group.keep {
                                        self.duplicates.selected.insert(file.path.clone());
                                    }
                                }
                            }
                            self.duplicates.groups = groups;
                            self.duplicates.done = true;
                        }
                        Ok(None) => {
                            self.duplicates.groups.clear();
                            self.duplicates.selected.clear();
                            self.duplicates.progress = (0, 0);
                        }
                        Err(text) => self.notice = Some(Notice::error(text)),
                    }
                }
                Message::Trashed(result) => {
                    keep = false;
                    self.duplicates.trashing = false;
                    self.duplicates.cancel = None;
                    match result {
                        Ok(outcome) => {
                            self.duplicates.clear();
                            self.refresh_history();
                            self.notice = Some(Notice::info(i18n::duplicates_done(
                                self.lang,
                                outcome.trashed,
                                outcome.bytes,
                                outcome.failed,
                            )));
                        }
                        Err(text) => self.notice = Some(Notice::error(text)),
                    }
                }
            }
        }
        if keep {
            self.duplicates.receiver = Some(receiver);
        }
    }

    pub fn ask_trash_duplicates(&mut self) {
        if self.duplicates.selected.is_empty() {
            self.notice = Some(Notice::info(
                i18n::t(self.lang, Key::DuplicatesNoSelection).to_string(),
            ));
            return;
        }
        self.confirm = Some(super::Confirm::Duplicates {
            files: self.duplicates.selected.len(),
            bytes: self.duplicates.selected_bytes(),
        });
    }

    pub fn trash_duplicates(&mut self, ctx: &egui::Context) {
        if self.duplicates.trashing || self.duplicates.selected.is_empty() {
            return;
        }
        self.duplicates.trashing = true;
        let groups = self.duplicates.groups.clone();
        let selected = self.duplicates.selected.clone();
        let lang = self.lang;
        let cancel = Arc::new(AtomicBool::new(false));
        self.duplicates.cancel = Some(cancel.clone());
        let (sender, receiver) = mpsc::channel();
        self.duplicates.receiver = Some(receiver);
        let repaint = ctx.clone();

        thread::spawn(move || {
            crate::priority::lower_thread();
            let result = duplicates::trash_selected(&groups, &selected, &cancel, |_, _| {});
            let _ = sender.send(Message::Trashed(result.map_err(|err| err.describe(lang))));
            repaint.request_repaint();
        });
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::palette(ui, &app.settings);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(i18n::t(app.lang, Key::DuplicatesTitle))
                    .size(22.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(i18n::t(app.lang, Key::DuplicatesIntro))
                    .size(13.0)
                    .color(palette.muted),
            );
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.duplicates.scanning {
                if widgets::ghost_button(ui, &palette, i18n::t(app.lang, Key::DuplicatesCancel))
                    .clicked()
                {
                    app.cancel_duplicates();
                }
            } else if widgets::accent_button(ui, &palette, i18n::t(app.lang, Key::FindDuplicates))
                .clicked()
            {
                app.start_duplicates(ui.ctx());
            }
        });
    });

    ui.add_space(14.0);

    if app.duplicates.scanning || app.duplicates.trashing {
        let (done, total) = app.duplicates.progress;
        let text = if app.duplicates.trashing {
            i18n::t(app.lang, Key::DuplicatesAction).to_string()
        } else {
            i18n::duplicates_progress(app.lang, done, total)
        };
        widgets::card(ui, &palette, |ui| {
            ui.label(RichText::new(text).size(13.0).color(palette.text));
            ui.add_space(6.0);
            let fraction = if total == 0 {
                0.0
            } else {
                done as f32 / total as f32
            };
            ui.add(egui::ProgressBar::new(fraction).desired_height(8.0));
            ui.add_space(4.0);
            widgets::small_muted(ui, &palette, i18n::t(app.lang, Key::DuplicatesScanning));
        });
        return;
    }

    if app.duplicates.groups.is_empty() {
        if app.duplicates.done {
            widgets::card(ui, &palette, |ui| {
                ui.label(
                    RichText::new(i18n::t(app.lang, Key::DuplicatesEmpty))
                        .size(13.0)
                        .color(palette.muted),
                );
            });
        }
        return;
    }

    ui.label(
        RichText::new(i18n::t(app.lang, Key::DuplicatesIdentical))
            .size(13.0)
            .color(palette.text),
    );
    ui.add_space(2.0);
    let total_groups = app.duplicates.groups.len();
    let total_files: usize = app
        .duplicates
        .groups
        .iter()
        .map(|group| group.files.len())
        .sum();
    widgets::small_muted(
        ui,
        &palette,
        &i18n::duplicates_found(
            app.lang,
            total_groups,
            total_files,
            app.duplicates.selected_bytes(),
        ),
    );
    ui.add_space(8.0);

    let mut rows: Vec<(usize, Option<usize>)> = Vec::new();
    for (group_index, group) in app.duplicates.groups.iter().enumerate() {
        rows.push((group_index, None));
        for file_index in 0..group.files.len() {
            rows.push((group_index, Some(file_index)));
        }
    }

    let row_height = 34.0;
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .max_height(420.0)
        .show_rows(ui, row_height, rows.len(), |ui, range| {
            for row in &rows[range] {
                let (group_index, file_index) = *row;
                let Some(group) = app.duplicates.groups.get(group_index) else {
                    continue;
                };
                let count = group.files.len();
                let group_size = group.size;
                let wasted = group.wasted();
                let keep_index = group.keep;
                let previous_keep = group.files.get(keep_index).map(|file| file.path.clone());
                let file = file_index.and_then(|index| group.files.get(index));
                let file_path = file.map(|file| file.path.clone());
                match file_index {
                    None => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{count} {} · {} · {}",
                                    i18n::t(app.lang, Key::FilesLabel),
                                    i18n::bytes_size(app.lang, group_size),
                                    modified_date(
                                        group.files.get(keep_index).map(|file| file.modified)
                                    )
                                ))
                                .size(13.0)
                                .strong()
                                .color(palette.text),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                widgets::small_muted(
                                    ui,
                                    &palette,
                                    &i18n::bytes_size(app.lang, wasted),
                                );
                            });
                        });
                    }
                    Some(file_index) => {
                        let Some(path) = file_path else {
                            continue;
                        };
                        let is_keep = keep_index == file_index;
                        let mut remove = app.duplicates.selected.contains(&path);
                        ui.horizontal(|ui| {
                            if is_keep {
                                widgets::pill(
                                    ui,
                                    &palette,
                                    i18n::t(app.lang, Key::DuplicatesKeep),
                                    palette.success,
                                );
                            } else if ui
                                .checkbox(&mut remove, "")
                                .on_hover_text(i18n::t(app.lang, Key::DuplicatesDelete))
                                .changed()
                            {
                                if remove {
                                    app.duplicates.selected.insert(path.clone());
                                } else {
                                    app.duplicates.selected.remove(&path);
                                }
                            }

                            let label = path.display().to_string();
                            ui.add(
                                egui::Label::new(
                                    RichText::new(label.clone()).size(13.0).color(palette.text),
                                )
                                .truncate(),
                            )
                            .on_hover_text(label);

                            if !is_keep
                                && widgets::link_button(
                                    ui,
                                    &palette,
                                    i18n::t(app.lang, Key::DuplicatesKeep),
                                )
                                .clicked()
                            {
                                if let Some(previous) = previous_keep.clone() {
                                    app.duplicates.selected.insert(previous);
                                }
                                app.duplicates.selected.remove(&path);
                                if let Some(group) = app.duplicates.groups.get_mut(group_index) {
                                    group.keep = file_index;
                                }
                            }
                        });
                    }
                }
            }
        });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        widgets::small_muted(
            ui,
            &palette,
            &format!(
                "{}: {}",
                i18n::t(app.lang, Key::DuplicatesFreed),
                i18n::bytes_size(app.lang, app.duplicates.selected_bytes())
            ),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::danger_button(ui, &palette, i18n::t(app.lang, Key::DuplicatesDelete))
                .clicked()
            {
                app.ask_trash_duplicates();
            }
        });
    });
}

pub fn open_from_summary(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    if !app.settings.advanced || app.settings.roots.is_empty() {
        return;
    }
    if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::FindDuplicates)).clicked() {
        app.start_duplicates(ui.ctx());
    }
}

fn modified_date(time: Option<std::time::SystemTime>) -> String {
    let Some(time) = time else {
        return String::new();
    };
    let parsed = time::OffsetDateTime::from(time);
    let local = crate::clock::offset()
        .map(|offset| parsed.to_offset(offset))
        .unwrap_or(parsed);
    format!(
        "{:04}-{:02}-{:02}",
        local.year(),
        u8::from(local.month()),
        local.day()
    )
}
