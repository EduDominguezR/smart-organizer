//! Historial: lotes organizados, con deshacer y exportacion.

use eframe::egui::{self, Align, Layout, RichText};

use super::App;
use super::theme;
use super::widgets;
use crate::i18n::{self, Key};
use crate::journal::{BatchSummary, format_stamp};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::palette(ui, &app.settings);
    let mut export = false;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(i18n::t(app.lang, Key::HistoryTitle))
                    .size(22.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(2.0);
            widgets::small_muted(
                ui,
                &palette,
                &i18n::files_label(app.lang, app.totals.files as usize),
            );
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let available = !app.history.is_empty();
            let response = widgets::ghost_button_enabled(
                ui,
                &palette,
                i18n::t(app.lang, Key::ExportCsv),
                available,
            );
            if !available {
                response.on_disabled_hover_text(i18n::t(app.lang, Key::SaveListEmptyHint));
            } else if response.clicked() {
                export = true;
            }
        });
    });

    ui.add_space(12.0);

    let batches: Vec<BatchSummary> = app.history.clone();

    if batches.is_empty() {
        widgets::empty_state(
            ui,
            &palette,
            i18n::t(app.lang, Key::HistoryEmptyTitle),
            i18n::t(app.lang, Key::HistoryEmpty),
            |_ui| {},
        );
        if export {
            app.export_csv();
        }
        return;
    }

    let row_height = 128.0;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, batches.len(), |ui, range| {
            for index in range {
                let Some(batch) = batches.get(index) else {
                    continue;
                };
                let mut undo = false;

                widgets::card(ui, &palette, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format_stamp(&batch.at))
                                        .size(13.0)
                                        .strong()
                                        .color(palette.text),
                                );
                                widgets::pill(
                                    ui,
                                    &palette,
                                    &i18n::files_label(app.lang, batch.files),
                                    palette.accent_text,
                                );
                                if batch.bytes > 0 {
                                    widgets::pill(
                                        ui,
                                        &palette,
                                        &i18n::bytes_size(app.lang, batch.bytes),
                                        palette.muted,
                                    );
                                }
                                if batch.trashed > 0 {
                                    widgets::pill(
                                        ui,
                                        &palette,
                                        &i18n::trashed_label(app.lang, batch.trashed),
                                        palette.muted,
                                    );
                                }
                                if batch.undone {
                                    widgets::pill(
                                        ui,
                                        &palette,
                                        i18n::t(app.lang, Key::Undone),
                                        palette.muted,
                                    );
                                }
                                if batch.review > 0 {
                                    widgets::pill(
                                        ui,
                                        &palette,
                                        &i18n::review_label(app.lang, batch.review),
                                        palette.error,
                                    );
                                }
                            });

                            ui.add_space(4.0);

                            ui.horizontal_wrapped(|ui| {
                                for destination in &batch.destinations {
                                    widgets::pill(ui, &palette, destination, palette.success);
                                }
                            });

                            if batch.review > 0 {
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(i18n::t(app.lang, Key::HistoryReviewHint))
                                        .size(12.0)
                                        .color(palette.error),
                                );
                            }
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !batch.undone
                                && batch.files > 0
                                && widgets::ghost_button(
                                    ui,
                                    &palette,
                                    i18n::t(app.lang, Key::HistoryUndo),
                                )
                                .clicked()
                            {
                                undo = true;
                            }
                        });
                    });
                });

                if undo {
                    app.ask_undo_batch(batch);
                }

                ui.add_space(8.0);
            }
        });

    if export {
        app.export_csv();
    }
}
