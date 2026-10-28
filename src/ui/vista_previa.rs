//! Vista previa: exactamente lo que se moveria, con casillas para excluir.

use std::path::Path;

use eframe::egui::{self, Align, Layout, RichText};

use super::App;
use super::theme::{self, Palette};
use super::widgets;
use crate::i18n::{self, Key};
use crate::organize::{Plan, PlanOp, PlannedItem};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::palette(ui, &app.settings);

    if app.settings.roots.is_empty() {
        widgets::empty_state(
            ui,
            &palette,
            i18n::t(app.lang, Key::EmptyHomeTitle),
            i18n::t(app.lang, Key::EmptyPreviewBody),
            |ui| {
                if widgets::accent_button(ui, &palette, i18n::t(app.lang, Key::AddFolder)).clicked()
                {
                    app.pick_folder();
                }
                ui.add_space(8.0);
                app.shortcut_buttons(ui, &palette);
            },
        );
        return;
    }

    if app.scanning && app.plan.is_none() {
        ui.add_space(48.0);
        ui.vertical_centered(|ui| {
            ui.add(egui::Spinner::new().size(22.0));
            ui.add_space(8.0);
            widgets::small_muted(ui, &palette, i18n::t(app.lang, Key::Scanning));
        });
        return;
    }

    let Some(mut plan) = app.plan.take() else {
        empty_plan(app, ui, &palette);
        return;
    };

    let mut rescan = false;
    let mut organize = false;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(i18n::t(app.lang, Key::PreviewTitle))
                    .size(22.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(2.0);

            let destinations = plan.destinations().join(", ");
            if !plan.items.is_empty() {
                widgets::small_muted(
                    ui,
                    &palette,
                    &i18n::preview_summary(app.lang, plan.selected_count(), &destinations),
                );
            }
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::ghost_button(ui, &palette, i18n::t(app.lang, Key::Rescan)).clicked() {
                rescan = true;
            }
            super::duplicados::open_from_summary(app, ui, &palette);
        });
    });

    ui.add_space(12.0);

    if plan.items.is_empty() {
        widgets::card(ui, &palette, |ui| {
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(i18n::t(app.lang, Key::EmptyNoPlan))
                        .size(16.0)
                        .color(palette.text),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(i18n::t(app.lang, Key::EmptyPlanBody))
                        .size(13.0)
                        .color(palette.muted),
                );
                ui.add_space(10.0);
                if widgets::ghost_button(ui, &palette, i18n::t(app.lang, Key::Rescan)).clicked() {
                    rescan = true;
                }
            });
            ui.add_space(16.0);
        });

        app.plan = Some(plan);
        if rescan {
            app.scan();
        }
        return;
    }

    // Barra de filtro y seleccion.
    ui.horizontal(|ui| {
        let mut all = plan.items.iter().all(|item| item.include);
        if ui
            .checkbox(&mut all, i18n::t(app.lang, Key::PreviewSelectAll))
            .changed()
        {
            for item in &mut plan.items {
                item.include = all;
            }
        }

        ui.add_space(8.0);
        widgets::text_input(
            ui,
            &palette,
            &mut app.filter,
            i18n::t(app.lang, Key::PreviewFilter),
            220.0,
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            widgets::small_muted(
                ui,
                &palette,
                &i18n::files_label(app.lang, plan.selected_count()),
            );
        });
    });

    ui.add_space(10.0);

    let filter = app.filter.to_lowercase();
    let row_height = theme::row_height(&app.settings);
    let visible: Vec<usize> = plan
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| filter.is_empty() || item.name.to_lowercase().contains(&filter))
        .map(|(index, _)| index)
        .collect();

    for note in plan_notes(app, &plan) {
        widgets::small_muted(ui, &palette, &note);
    }

    // Rectangulos explicitos: la tabla ocupa todo el alto y el pie siempre cabe.
    let area = ui.available_rect_before_wrap();
    let footer_height = 46.0;
    let card_rect = egui::Rect::from_min_max(
        area.min,
        egui::pos2(area.right(), area.bottom() - footer_height - 10.0),
    );
    let footer_rect = egui::Rect::from_min_max(
        egui::pos2(area.left(), area.bottom() - footer_height),
        area.max,
    );
    let scroll_height = (card_rect.height() - 64.0).max(120.0);

    ui.scope_builder(egui::UiBuilder::new().max_rect(card_rect), |ui| {
        widgets::card(ui, &palette, |ui| {
            let width = ui.available_width();
            let check_width = 26.0;
            let destination_width = (width * 0.28).clamp(150.0, 260.0);
            let size_width = 74.0;
            let name_width =
                (width - check_width - destination_width - size_width - 30.0).max(140.0);

            ui.horizontal(|ui| {
                ui.allocate_exact_size(egui::vec2(check_width, 14.0), egui::Sense::hover());
                column_label(ui, &palette, app, name_width, Key::ColFiles);
                column_label(ui, &palette, app, destination_width, Key::ColDestination);
                column_label(ui, &palette, app, size_width, Key::SizeColumn);
            });

            ui.add_space(4.0);
            widgets::divider(ui, &palette);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(scroll_height)
                .show_rows(ui, row_height + 2.0, visible.len(), |ui, range| {
                    for position in range {
                        let Some(index) = visible.get(position).copied() else {
                            continue;
                        };
                        let Some(item) = plan.items.get_mut(index) else {
                            continue;
                        };

                        ui.horizontal(|ui| {
                            let (check_rect, _) = ui.allocate_exact_size(
                                egui::vec2(check_width, row_height),
                                egui::Sense::hover(),
                            );
                            ui.put(
                                egui::Rect::from_center_size(
                                    check_rect.center(),
                                    egui::vec2(18.0, 18.0),
                                ),
                                egui::Checkbox::new(&mut item.include, ""),
                            );

                            ui.allocate_ui_with_layout(
                                egui::vec2(name_width, row_height),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.set_min_width(name_width);
                                    ui.add_space(2.0);
                                    truncated(ui, &item.name, 13.0, palette.text, true);
                                    let parent = item
                                        .from
                                        .parent()
                                        .map(|parent| parent.display().to_string())
                                        .unwrap_or_default();
                                    truncated(ui, &parent, 11.0, palette.muted, false);
                                },
                            );

                            ui.allocate_ui_with_layout(
                                egui::vec2(destination_width, row_height),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.set_min_width(destination_width);
                                    if item.op == PlanOp::Trash {
                                        widgets::pill(
                                            ui,
                                            &palette,
                                            i18n::t(app.lang, Key::Trash),
                                            palette.error,
                                        );
                                    } else {
                                        let label = category_label(app, item);
                                        widgets::pill(ui, &palette, &label, palette.accent_text);
                                        if item.op == PlanOp::Copy {
                                            widgets::small_muted(
                                                ui,
                                                &palette,
                                                i18n::t(app.lang, Key::ActionCopy),
                                            );
                                        }
                                        if let Some(to) = &item.to {
                                            let folder = destination_folder(to, &plan.roots);
                                            if folder != label {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(folder)
                                                            .size(13.0)
                                                            .color(palette.muted),
                                                    )
                                                    .truncate(),
                                                );
                                            }
                                        }
                                    }
                                },
                            );

                            ui.allocate_ui_with_layout(
                                egui::vec2(size_width, row_height),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.set_min_width(size_width);
                                    ui.label(
                                        RichText::new(i18n::bytes_size(app.lang, item.size))
                                            .size(13.0)
                                            .color(palette.muted),
                                    );
                                },
                            );
                        });

                        ui.add_space(2.0);
                    }
                });
        });
    });

    ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let count = plan.selected_count();
            let label = format!("{} ({count})", i18n::t(app.lang, Key::OrganizeNow));
            if widgets::accent_button_enabled(ui, &palette, &label, count > 0 && !app.organizing)
                .clicked()
            {
                organize = true;
            }
        });
    });

    app.plan = Some(plan);

    if rescan {
        app.scan();
    }
    if organize {
        app.ask_organize();
    }
}

fn empty_plan(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    widgets::empty_state(
        ui,
        palette,
        i18n::t(app.lang, Key::EmptyNoPlan),
        i18n::t(app.lang, Key::EmptyPlanBody),
        |ui| {
            ui.horizontal(|ui| {
                if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::Rescan)).clicked() {
                    app.scan();
                }
                super::duplicados::open_from_summary(app, ui, palette);
            });
        },
    );
}

fn plan_notes(app: &App, plan: &Plan) -> Vec<String> {
    let mut notes = Vec::new();

    if plan.left_alone > 0 {
        notes.push(i18n::skipped_note(app.lang, plan.left_alone));
    }
    if plan.trash_count() > 0 {
        notes.push(i18n::t(app.lang, Key::TrashNote).to_string());
    }
    if plan.copy_count() > 0 {
        notes.push(i18n::t(app.lang, Key::UndoCopyNote).to_string());
    }

    notes
}

fn relative(path: &Path, roots: &[std::path::PathBuf]) -> String {
    for root in roots {
        if let Ok(relative) = path.strip_prefix(root) {
            return relative.display().to_string();
        }
    }
    path.display().to_string()
}

fn category_label(app: &App, item: &PlannedItem) -> String {
    let categories = app
        .settings
        .roots
        .iter()
        .find(|root| root.path == item.root)
        .and_then(|root| root.overrides.categories.as_ref())
        .unwrap_or(&app.settings.categories);

    categories
        .iter()
        .find(|category| category.name == item.category)
        .map(|category| category.folder.clone())
        .unwrap_or_else(|| item.category.clone())
}

fn destination_folder(path: &Path, roots: &[std::path::PathBuf]) -> String {
    let relative = relative(path, roots);

    match Path::new(&relative).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.display().to_string(),
        _ => relative,
    }
}

fn column_label(ui: &mut egui::Ui, palette: &Palette, app: &App, width: f32, key: Key) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, 14.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(width);
            widgets::section_label(ui, palette, i18n::t(app.lang, key));
        },
    );
}

fn truncated(ui: &mut egui::Ui, text: &str, size: f32, color: egui::Color32, strong: bool) {
    let mut text = RichText::new(text).size(size).color(color);
    if strong {
        text = text.strong();
    }
    ui.add(egui::Label::new(text).truncate());
}
