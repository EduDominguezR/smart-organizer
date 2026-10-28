//! Pantalla de inicio: estado, totales, actividad reciente y categorias.

use eframe::egui::{self, Align, Layout, RichText};

use super::theme::{self, Palette};
use super::widgets;
use super::{App, Tab, last_run_label};
use crate::config;
use crate::i18n::{self, Key};
use crate::journal::{BatchSummary, format_stamp};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::palette(ui, &app.settings);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if app.settings.roots.is_empty() {
                empty(app, ui, &palette);
                return;
            }

            header(app, ui, &palette);
            ui.add_space(18.0);
            if !app.settings.advanced {
                auto_organize(app, ui, &palette);
                ui.add_space(14.0);
            }
            stats(app, ui, &palette);
            ui.add_space(14.0);
            banner(app, ui, &palette);
            ui.add_space(14.0);
            recent(app, ui, &palette);
            ui.add_space(14.0);
            chart(app, ui, &palette);
            ui.add_space(6.0);
        });
}

fn empty(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    ui.add_space(34.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(i18n::t(app.lang, Key::EmptyHomeTitle))
                .size(22.0)
                .strong()
                .color(palette.text),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(i18n::t(app.lang, Key::EmptyHomeBody))
                .size(14.0)
                .color(palette.muted),
        );
        ui.add_space(16.0);
        if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::AddFolder)).clicked() {
            app.pick_folder();
        }
        ui.add_space(10.0);
        app.shortcut_buttons(ui, palette);
    });

    ui.add_space(24.0);
    widgets::card(ui, palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (step, key) in [
                (1, Key::StepChooseFolder),
                (2, Key::StepReviewFiles),
                (3, Key::StepMoveFiles),
            ] {
                let current = step == 1;
                if current {
                    widgets::pill(ui, palette, &step.to_string(), palette.accent);
                } else {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                    ui.painter().circle_stroke(
                        rect.center(),
                        10.0,
                        egui::Stroke::new(1.0, palette.border),
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        step.to_string(),
                        egui::FontId::proportional(12.0),
                        palette.muted,
                    );
                }
                ui.label(
                    RichText::new(i18n::t(app.lang, key))
                        .size(13.0)
                        .color(if current { palette.text } else { palette.muted }),
                );
                if step < 3 {
                    ui.label(RichText::new("→").size(13.0).color(palette.muted));
                }
            }
        });
    });
}

fn header(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            let pending = app
                .plan
                .as_ref()
                .map(|plan| plan.selected().count())
                .unwrap_or(0);
            let title = if app.scanning {
                i18n::t(app.lang, Key::Scanning).to_string()
            } else if pending > 0 {
                i18n::files_ready(app.lang, pending)
            } else {
                i18n::t(app.lang, Key::HomeReady).to_string()
            };
            ui.label(
                RichText::new(&title)
                    .size(24.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(2.0);
            widgets::small_muted(ui, palette, &last_run_label(app));
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::Rescan)).clicked() {
                app.scan();
            }
            super::duplicados::open_from_summary(app, ui, palette);
            if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::AddFolder)).clicked() {
                app.pick_folder();
            }
        });
    });

    ui.add_space(10.0);

    ui.horizontal_wrapped(|ui| {
        for root in &app.settings.roots {
            let name = root
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.path.display().to_string());
            widgets::folder_chip(ui, palette, &name);
        }
    });
}

fn auto_organize(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    widgets::card(ui, palette, |ui| {
        let running = app.watcher_running();
        let mut enabled = running;
        ui.horizontal(|ui| {
            if widgets::switch(
                ui,
                palette,
                &mut enabled,
                i18n::t(app.lang, Key::AutoOrganize),
                app.lang,
            )
            .changed()
            {
                if enabled {
                    if !app.settings.advanced {
                        app.settings.watch.debounce_ms = 1200;
                        app.settings.watch.stability_ms = 1500;
                        app.settings.watch.recent_guard_minutes = 1;
                        app.settings.watch.schedule_minutes = 0;
                    }
                    app.start_watcher();
                } else {
                    app.stop_watcher();
                }
            }
            if (running || app.watch_failed || app.watch_retry_at.is_some())
                && widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::RestartWatch))
                    .clicked()
            {
                app.restart_watcher();
            }
        });
        if app.watch_failed {
            ui.add_space(4.0);
            ui.label(
                RichText::new(i18n::t(app.lang, Key::WatchFailed))
                    .size(13.0)
                    .color(palette.error),
            );
        }
        ui.add_space(2.0);
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::AutoOrganizeHint));
    });
}

fn stats(app: &App, ui: &mut egui::Ui, palette: &Palette) {
    let totals = &app.totals;
    let rules = app.settings.enabled_rules().to_string();
    let files = totals.files.to_string();
    let bytes = i18n::bytes_size(app.lang, totals.bytes);
    let organized_hint = i18n::t(app.lang, Key::FilesLabel);

    ui.columns(3, |columns| {
        widgets::card(&mut columns[0], palette, |ui| {
            widgets::stat(
                ui,
                palette,
                i18n::t(app.lang, Key::TotalOrganized),
                &files,
                organized_hint,
            );
        });
        widgets::card(&mut columns[1], palette, |ui| {
            widgets::stat(ui, palette, i18n::t(app.lang, Key::SpaceFreed), &bytes, "");
        });
        widgets::card(&mut columns[2], palette, |ui| {
            widgets::stat(ui, palette, i18n::t(app.lang, Key::ActiveRules), &rules, "");
        });
    });
}

fn banner(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    let ready = app
        .plan
        .as_ref()
        .map(|plan| (plan.selected_count(), plan.destinations().join(", ")))
        .filter(|(count, _)| *count > 0);

    if let Some((files, destinations)) = ready {
        widgets::card(ui, palette, |ui| {
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot.center(), 4.0, palette.accent);

                ui.label(
                    RichText::new(i18n::plan_ready(app.lang, files, &destinations))
                        .size(14.0)
                        .color(palette.text),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::TabPreview))
                        .clicked()
                    {
                        app.tab = Tab::Vista;
                    }
                });
            });
        });
        return;
    }

    widgets::card(ui, palette, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if app.scanning {
                    i18n::t(app.lang, Key::Scanning)
                } else {
                    i18n::t(app.lang, Key::EmptyNoPlan)
                })
                .size(13.0)
                .color(palette.muted),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::Rescan)).clicked() {
                    app.scan();
                }
            });
        });
    });
}

fn recent(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    let batches: Vec<BatchSummary> = app.history.iter().take(5).cloned().collect();

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::RecentActivity), |ui| {
        if batches.is_empty() {
            widgets::small_muted(ui, palette, i18n::t(app.lang, Key::HistoryEmpty));
            return;
        }

        egui::Grid::new("actividad-reciente")
            .num_columns(3)
            .spacing([16.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                for key in [Key::ColTime, Key::ColFiles, Key::ColDestination] {
                    widgets::section_label(ui, palette, i18n::t(app.lang, key));
                }
                ui.end_row();

                for batch in &batches {
                    ui.label(
                        RichText::new(format_stamp(&batch.at))
                            .size(13.0)
                            .color(palette.text),
                    );
                    ui.label(
                        RichText::new(i18n::files_label(app.lang, batch.files))
                            .size(13.0)
                            .color(palette.muted),
                    );
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(batch.destinations.join(", "))
                                .size(13.0)
                                .color(palette.muted),
                        );
                        if batch.undone {
                            widgets::pill(
                                ui,
                                palette,
                                i18n::t(app.lang, Key::Undone),
                                palette.muted,
                            );
                        }
                    });
                    ui.end_row();
                }
            });

        ui.add_space(6.0);
        if widgets::link_button(ui, palette, i18n::t(app.lang, Key::SeeAll)).clicked() {
            app.tab = Tab::Historial;
        }
    });
}

fn chart(app: &App, ui: &mut egui::Ui, palette: &Palette) {
    let data: Vec<(String, usize)> = app
        .totals
        .categories
        .iter()
        .map(|(name, count)| (name.clone(), *count as usize))
        .collect();

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::CategoryStats), |ui| {
        let color_of = |name: &str| {
            app.settings
                .category_color(name)
                .and_then(|hex| config::normalize_color(&hex))
                .map(|hex| theme::hex(&hex, palette.accent))
                .unwrap_or(palette.accent)
        };
        widgets::bar_chart(
            ui,
            palette,
            &data,
            6,
            i18n::t(app.lang, Key::NoData),
            color_of,
        );
    });
}
