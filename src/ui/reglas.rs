//! Editor de reglas y probador. Se usa en Ajustes y en la pantalla Organizar.

use eframe::egui::{self, RichText};

use super::theme::{self, ACCENTS, Palette};
use super::widgets;
use super::{App, Notice, RuleDraft, Scope};
use crate::config::{self, Action, Condition, DateSource, Rule, RuleSubfolder};
use crate::i18n::{self, Key};
use crate::rules::{self, Target};

const KINDS: usize = 14;

pub fn editor(app: &mut App, ui: &mut egui::Ui, palette: &Palette, scope: Scope, dirty: &mut bool) {
    let mut rules = app.scope_rules(scope).cloned().unwrap_or_default();

    if groups(app, ui, palette, scope, dirty) {
        return;
    }

    ui.add_space(12.0);

    let mut changed = false;
    rules_list(app, ui, palette, scope, &mut rules, &mut changed);
    ui.add_space(12.0);
    simple_rule_form(app, ui, palette, scope, &mut rules, &mut changed);
    ui.add_space(12.0);

    if changed {
        app.set_scope_rules(scope, rules);
        *dirty = true;
    }

    live_preview(app, ui, palette, scope);

    ui.add_space(8.0);
    egui::CollapsingHeader::new(i18n::t(app.lang, Key::RuleTester))
        .id_salt(("rule-tester", rule_scope_id(scope)))
        .show(ui, |ui| {
            tester(app, ui, palette, scope);
        });
}

fn rule_scope_id(scope: Scope) -> usize {
    match scope {
        Scope::Global => usize::MAX,
        Scope::Root(index) => index,
    }
}

/// Dibuja los grupos y devuelve true si alguna accion ya se aplico y conviene
/// esperar al siguiente repintado para no pisar el estado nuevo.
fn groups(
    app: &mut App,
    ui: &mut egui::Ui,
    palette: &Palette,
    scope: Scope,
    dirty: &mut bool,
) -> bool {
    let Some(mut categories) = app.scope_categories(scope).cloned() else {
        return false;
    };
    let lang = app.lang;
    let mut changed = false;
    let mut action: Option<GroupAction> = None;
    let category_count = categories.len();
    let owners: Vec<(String, Vec<String>)> = categories
        .iter()
        .map(|category| (category.name.clone(), category.extensions.clone()))
        .collect();

    widgets::card_titled(ui, palette, i18n::t(lang, Key::GroupTitle), |ui| {
        ui.horizontal(|ui| {
            widgets::help_icon(ui, palette, i18n::t(lang, Key::GroupHint));
            widgets::small_muted(ui, palette, i18n::t(lang, Key::GroupHint));
        });
        ui.add_space(6.0);

        for (index, category) in categories.iter_mut().enumerate() {
            let expanded = app.expanded_group == Some(index);
            ui.horizontal(|ui| {
                widgets::folder_icon(ui, palette);
                let title = category.name.clone();
                let accessible_title = title.clone();
                ui.vertical(|ui| {
                    ui.set_max_width((ui.available_width() - 190.0).max(120.0));
                    let name = ui.add(
                        egui::Label::new(
                            RichText::new(title).size(14.0).strong().color(palette.text),
                        )
                        .sense(egui::Sense::click())
                        .selectable(false),
                    );
                    if name.clicked() {
                        app.expanded_group = if expanded { None } else { Some(index) };
                    }
                    let state = if expanded {
                        i18n::t(lang, Key::StateExpanded)
                    } else {
                        i18n::t(lang, Key::StateCollapsed)
                    };
                    name.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::DisclosureTriangle,
                            true,
                            expanded,
                            format!("{accessible_title}, {state}"),
                        )
                    });
                    if name.has_focus() {
                        ui.painter().rect_stroke(
                            name.rect.shrink(1.0),
                            egui::CornerRadius::same(4),
                            egui::Stroke::new(1.0, palette.accent),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let count = i18n::group_count(lang, category.extensions.len());
                    let summary = i18n::file_types_summary(lang, &category.extensions);
                    let details = if summary.is_empty() {
                        count
                    } else {
                        format!("{count} · {summary}")
                    };
                    ui.add(
                        egui::Label::new(RichText::new(details).size(13.0).color(palette.muted))
                            .truncate(),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let menu = widgets::menu_button(ui, palette);
                    egui::Popup::menu(&menu).show(|ui| {
                        if ui
                            .add_enabled(index > 0, egui::Button::new(i18n::t(lang, Key::RuleUp)))
                            .clicked()
                        {
                            action = Some(GroupAction::Up(index));
                        }
                        if ui
                            .add_enabled(
                                index + 1 < category_count,
                                egui::Button::new(i18n::t(lang, Key::RuleDown)),
                            )
                            .clicked()
                        {
                            action = Some(GroupAction::Down(index));
                        }
                        let delete = ui.add_enabled(
                            !category.fallback,
                            egui::Button::new(i18n::t(lang, Key::GroupDelete)),
                        );
                        if delete.clicked() {
                            action = Some(GroupAction::Delete(index));
                        }
                        if category.fallback {
                            delete.on_disabled_hover_text(i18n::t(lang, Key::GroupNoDelete));
                        }
                    });
                    let active =
                        ui.checkbox(&mut category.enabled, i18n::t(lang, Key::GroupActive));
                    if active.changed() {
                        changed = true;
                    }
                });
            });

            if expanded {
                ui.add_space(6.0);
                egui::Grid::new(("grupo-detalle", index))
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(i18n::t(lang, Key::GroupName))
                                .size(13.0)
                                .color(palette.muted),
                        );
                        let old_name = category.name.clone();
                        if widgets::text_input(ui, palette, &mut category.name, "", 240.0).changed()
                        {
                            if category.folder == old_name {
                                category.folder = config::sanitize_folder(&category.name);
                            }
                            changed = true;
                        }
                        ui.end_row();

                        ui.label(
                            RichText::new(i18n::t(lang, Key::GroupTypes))
                                .size(13.0)
                                .color(palette.muted),
                        );
                        ui.vertical(|ui| {
                            if category.extensions.is_empty() {
                                widgets::small_muted(
                                    ui,
                                    palette,
                                    i18n::t(lang, Key::GroupTypesEmpty),
                                );
                            }
                            let mut remove_type = None;
                            ui.horizontal_wrapped(|ui| {
                                for (type_index, extension) in
                                    category.extensions.iter().enumerate()
                                {
                                    let raw = extension.trim().trim_start_matches('.');
                                    let text = format!(".{raw} ×");
                                    let chip = ui.add(
                                        egui::Button::new(
                                            RichText::new(text)
                                                .size(13.0)
                                                .color(palette.accent_text),
                                        )
                                        .fill(palette.tint(palette.accent))
                                        .stroke(egui::Stroke::NONE)
                                        .corner_radius(egui::CornerRadius::same(4))
                                        .min_size(egui::vec2(0.0, 28.0)),
                                    );
                                    if chip.clicked() {
                                        remove_type = Some(type_index);
                                    }
                                }
                            });
                            if let Some(type_index) = remove_type {
                                category.extensions.remove(type_index);
                                changed = true;
                            }
                            ui.horizontal_wrapped(|ui| {
                                widgets::text_input(
                                    ui,
                                    palette,
                                    &mut app.draft_type,
                                    i18n::t(lang, Key::GroupTypeHint),
                                    110.0,
                                );
                                if widgets::ghost_button(
                                    ui,
                                    palette,
                                    &format!("+ {}", i18n::t(lang, Key::GroupAddType)),
                                )
                                .clicked()
                                {
                                    let extension = config::normalize_extension(&app.draft_type);
                                    if !extension.is_empty() {
                                        if category.extensions.contains(&extension) {
                                            app.draft_type.clear();
                                        } else if let Some((owner, _)) =
                                            owners.iter().enumerate().find(|(owner, (_, types))| {
                                                *owner != index && types.contains(&extension)
                                            })
                                        {
                                            let group = owners[owner].0.clone();
                                            app.notice = Some(Notice::error(
                                                i18n::t(lang, Key::GroupTypeInUse)
                                                    .replace("{group}", &group),
                                            ));
                                        } else {
                                            category.extensions.push(extension);
                                            category.extensions.sort();
                                            category.extensions.dedup();
                                            app.draft_type.clear();
                                            changed = true;
                                        }
                                    }
                                }
                            });
                            if !category.enabled {
                                widgets::small_muted(
                                    ui,
                                    palette,
                                    i18n::t(lang, Key::GroupPausedHint),
                                );
                            }
                        });
                        ui.end_row();

                        ui.label(
                            RichText::new(i18n::t(lang, Key::GroupFolder))
                                .size(13.0)
                                .color(palette.muted),
                        );
                        ui.label(
                            RichText::new(&category.folder)
                                .size(13.0)
                                .color(palette.text),
                        );
                        ui.end_row();

                        ui.label(
                            RichText::new(i18n::t(lang, Key::GroupColor))
                                .size(13.0)
                                .color(palette.muted),
                        );
                        ui.horizontal_wrapped(|ui| {
                            let accent = app.settings.accent.clone();
                            let selected = category
                                .color
                                .as_deref()
                                .unwrap_or(accent.as_str())
                                .to_string();
                            if widgets::link_button(
                                ui,
                                palette,
                                i18n::t(lang, Key::GroupColorAccent),
                            )
                            .clicked()
                            {
                                category.color = None;
                                changed = true;
                            }
                            for choice in ACCENTS {
                                if theme::accent_swatch(
                                    ui,
                                    palette,
                                    choice.value,
                                    selected.eq_ignore_ascii_case(choice.value),
                                )
                                .clicked()
                                {
                                    category.color = Some(choice.value.to_string());
                                    changed = true;
                                }
                            }
                        });
                        ui.end_row();
                    });

                egui::CollapsingHeader::new(i18n::t(lang, Key::GroupAdvanced))
                    .id_salt(("grupo-avanzado", index))
                    .show(ui, |ui| {
                        widgets::small_muted(ui, palette, i18n::t(lang, Key::GroupFolderHint));
                        let folder_changed = widgets::text_input(
                            ui,
                            palette,
                            &mut category.folder,
                            i18n::t(lang, Key::GroupFolder),
                            220.0,
                        )
                        .changed();
                        if folder_changed {
                            category.folder = config::sanitize_folder(&category.folder);
                            changed = true;
                        }
                    });
                ui.add_space(6.0);
            }
            widgets::divider(ui, palette);
        }

        widgets::small_muted(ui, palette, i18n::t(lang, Key::GroupActiveHint));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if widgets::ghost_button(ui, palette, i18n::t(lang, Key::GroupAdd)).clicked() {
                action = Some(GroupAction::Add);
            }
            if widgets::link_button(ui, palette, i18n::t(lang, Key::GroupRestore)).clicked() {
                action = Some(GroupAction::Restore);
            }
        });
    });

    if let Some(action) = action {
        match action {
            GroupAction::Add => app.add_group(scope),
            GroupAction::Delete(index) => app.delete_group(scope, index),
            GroupAction::Up(index) => app.move_group(scope, index, -1),
            GroupAction::Down(index) => app.move_group(scope, index, 1),
            GroupAction::Restore => app.restore_groups(scope),
        }
        *dirty = true;
        return true;
    }

    if changed {
        for (index, category) in categories.iter_mut().enumerate() {
            if category.name.trim().is_empty() {
                category.name =
                    i18n::t(lang, Key::GroupNewName).replace("{number}", &(index + 1).to_string());
            }
            if category.folder.trim().is_empty() {
                category.folder = config::sanitize_folder(&category.name);
            }
        }
        app.set_scope_categories(scope, categories);
        *dirty = true;
    }
    false
}

enum GroupAction {
    Add,
    Delete(usize),
    Up(usize),
    Down(usize),
    Restore,
}

fn rules_list(
    app: &mut App,
    ui: &mut egui::Ui,
    palette: &Palette,
    scope: Scope,
    rules: &mut Vec<Rule>,
    changed: &mut bool,
) {
    let lang = app.lang;
    widgets::card_titled(ui, palette, i18n::t(lang, Key::RuleListTitle), |ui| {
        ui.horizontal(|ui| {
            widgets::help_icon(ui, palette, i18n::t(lang, Key::TooltipSpecialRule));
        });
        ui.add_space(4.0);
        if rules.is_empty() {
            widgets::small_muted(ui, palette, i18n::t(lang, Key::NoRules));
            return;
        }

        let mut edit: Option<usize> = None;
        let mut remove: Option<usize> = None;
        let mut up: Option<usize> = None;
        let mut down: Option<usize> = None;
        let mut toggle: Option<(usize, bool)> = None;
        let mut undo_snapshot: Option<Vec<Rule>> = None;
        let count = rules.len();

        for (index, rule) in rules.iter().enumerate() {
            ui.horizontal(|ui| {
                let mut enabled = rule.enabled;
                if ui
                    .checkbox(&mut enabled, i18n::t(lang, Key::RuleEnabled))
                    .changed()
                {
                    toggle = Some((index, enabled));
                }
                ui.vertical(|ui| {
                    ui.set_max_width((ui.available_width() - 50.0).max(120.0));
                    ui.label(
                        RichText::new(&rule.name)
                            .size(14.0)
                            .strong()
                            .color(palette.text),
                    );
                    let summary = if rule.conditions.is_empty() {
                        i18n::t(lang, Key::RuleConditionsEmpty).to_string()
                    } else {
                        rule.conditions
                            .iter()
                            .map(|condition| condition_text(lang, condition))
                            .collect::<Vec<_>>()
                            .join(" · ")
                    };
                    widgets::small_muted(ui, palette, &summary);
                    ui.label(
                        RichText::new(action_text(lang, rule))
                            .size(13.0)
                            .color(palette.muted),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let menu = widgets::menu_button(ui, palette);
                    egui::Popup::menu(&menu).show(|ui| {
                        if ui.button(i18n::t(lang, Key::RuleEdit)).clicked() {
                            edit = Some(index);
                        }
                        if ui
                            .add_enabled(index > 0, egui::Button::new(i18n::t(lang, Key::RuleUp)))
                            .clicked()
                        {
                            up = Some(index);
                        }
                        if ui
                            .add_enabled(
                                index + 1 < count,
                                egui::Button::new(i18n::t(lang, Key::RuleDown)),
                            )
                            .clicked()
                        {
                            down = Some(index);
                        }
                        if ui
                            .button(
                                RichText::new(i18n::t(lang, Key::RuleDelete)).color(palette.error),
                            )
                            .clicked()
                        {
                            remove = Some(index);
                        }
                    });
                });
            });
            ui.add_space(6.0);
            widgets::divider(ui, palette);
        }

        if let Some((index, enabled)) = toggle {
            rules[index].enabled = enabled;
            *changed = true;
        }
        if let Some(index) = edit {
            app.rule_draft = RuleDraft::from_rule(&rules[index]);
            app.rule_error = None;
        }
        if let Some(index) = remove {
            undo_snapshot = Some(rules.clone());
            let removed = rules[index].id;
            let folder = rule_folder(&rules[index].action).map(str::to_string);
            rules.remove(index);
            if app.rule_draft.editing == Some(removed) {
                app.rule_draft = RuleDraft::default();
            }
            *changed = true;
            if let Some(folder) = folder {
                let exists = app
                    .scope_roots(scope)
                    .iter()
                    .any(|root| root.join(&folder).exists());
                if exists {
                    app.set_scope_rules(scope, rules.clone());
                    app.leave_folder_content(scope, &folder);
                }
            }
        }
        if let Some(index) = up {
            undo_snapshot.get_or_insert_with(|| rules.clone());
            rules.swap(index, index - 1);
            *changed = true;
        }
        if let Some(index) = down {
            undo_snapshot.get_or_insert_with(|| rules.clone());
            rules.swap(index, index + 1);
            *changed = true;
        }
        if let Some(previous) = undo_snapshot {
            app.undo_change = Some(super::UndoChange::Rules {
                scope,
                rules: previous,
            });
        }
    });
}

fn simple_rule_form(
    app: &mut App,
    ui: &mut egui::Ui,
    palette: &Palette,
    scope: Scope,
    rules: &mut Vec<Rule>,
    changed: &mut bool,
) {
    let lang = app.lang;
    let title = if app.rule_draft.editing.is_some() {
        i18n::t(lang, Key::RuleEdit)
    } else {
        i18n::t(lang, Key::RuleAdd)
    };

    widgets::card_titled(ui, palette, title, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(i18n::t(lang, Key::RuleWhen))
                    .size(13.0)
                    .color(palette.muted),
            );
            egui::ComboBox::from_id_salt("regla-simple-cuando")
                .selected_text(simple_kind_label(lang, app.rule_draft.simple_kind))
                .width(190.0)
                .show_ui(ui, |ui| {
                    for kind in [2usize, 0, 5, 7] {
                        ui.selectable_value(
                            &mut app.rule_draft.simple_kind,
                            kind,
                            simple_kind_label(lang, kind),
                        );
                    }
                });
            let hint = match app.rule_draft.simple_kind {
                0 => i18n::t(lang, Key::GroupTypeHint),
                5 => i18n::t(lang, Key::RuleExampleSize),
                7 => i18n::t(lang, Key::RuleExampleDays),
                _ => i18n::t(lang, Key::RuleExampleName),
            };
            widgets::text_input(ui, palette, &mut app.rule_draft.simple_value, hint, 150.0);
            ui.label(
                RichText::new(i18n::t(lang, Key::RuleThen))
                    .size(13.0)
                    .color(palette.muted),
            );
            ui.label(
                RichText::new(i18n::t(lang, Key::RuleDestination))
                    .size(13.0)
                    .color(palette.muted),
            );

            let folders: Vec<String> = app
                .scope_categories(scope)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|category| category.enabled)
                .map(|category| category.folder)
                .collect();
            if folders.is_empty() {
                widgets::text_input(
                    ui,
                    palette,
                    &mut app.rule_draft.target,
                    i18n::t(lang, Key::RuleDestination),
                    180.0,
                );
            } else {
                let selected = if app.rule_draft.target.is_empty() {
                    let first = folders[0].clone();
                    app.rule_draft.target = first.clone();
                    first
                } else {
                    app.rule_draft.target.clone()
                };
                egui::ComboBox::from_id_salt("regla-simple-destino")
                    .selected_text(selected.clone())
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for folder in &folders {
                            if ui
                                .selectable_value(
                                    &mut app.rule_draft.target,
                                    folder.clone(),
                                    folder,
                                )
                                .clicked()
                            {
                                app.rule_draft.action = 0;
                            }
                        }
                    });
                if app.rule_draft.target.is_empty() {
                    app.rule_draft.target = selected;
                }
            }
        });

        if let Some(error) = app.rule_error.as_deref() {
            ui.add_space(6.0);
            ui.label(RichText::new(error).size(13.0).color(palette.error));
        }

        ui.add_space(6.0);
        widgets::small_muted(ui, palette, i18n::t(lang, Key::RuleSpecialHint));

        egui::CollapsingHeader::new(i18n::t(lang, Key::RuleMoreOptions))
            .id_salt("regla-mas-opciones")
            .show(ui, |ui| {
                widgets::key_value(ui, palette, i18n::t(lang, Key::RuleName), "");
                widgets::text_input(ui, palette, &mut app.rule_draft.name, "", 280.0);

                ui.add_space(6.0);
                widgets::section_label(ui, palette, i18n::t(lang, Key::ConditionsTitle));
                let mut remove_condition: Option<usize> = None;
                for (index, condition) in app.rule_draft.conditions.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        widgets::pill(
                            ui,
                            palette,
                            &condition_text(lang, condition),
                            palette.accent_text,
                        );
                        if widgets::link_button(ui, palette, i18n::t(lang, Key::Remove)).clicked() {
                            remove_condition = Some(index);
                        }
                    });
                }
                if let Some(index) = remove_condition {
                    app.rule_draft.conditions.remove(index);
                }
                ui.horizontal_wrapped(|ui| {
                    let previous = app.rule_draft.condition_kind;
                    egui::ComboBox::from_id_salt("regla-condicion")
                        .selected_text(kind_label(lang, app.rule_draft.condition_kind))
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for index in 0..KINDS {
                                ui.selectable_value(
                                    &mut app.rule_draft.condition_kind,
                                    index,
                                    kind_label(lang, index),
                                );
                            }
                        });
                    if previous != app.rule_draft.condition_kind {
                        app.rule_draft.condition_value.clear();
                        app.rule_draft.condition_unit = 1;
                    }
                    condition_value_input(app, ui, palette);
                    if widgets::ghost_button(ui, palette, i18n::t(lang, Key::ConditionAdd))
                        .clicked()
                    {
                        match build_condition(app) {
                            Ok(condition) => {
                                app.rule_draft.conditions.push(condition);
                                app.rule_draft.condition_value.clear();
                            }
                            Err(message) => app.notice = Some(Notice::error(message)),
                        }
                    }
                });

                ui.add_space(6.0);
                widgets::divider(ui, palette);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(i18n::t(lang, Key::RuleAction))
                            .size(13.0)
                            .color(palette.muted),
                    );
                    egui::ComboBox::from_id_salt("regla-accion")
                        .selected_text(action_label(lang, app.rule_draft.action))
                        .width(190.0)
                        .show_ui(ui, |ui| {
                            for index in (0..7).filter(|index| *index != 2) {
                                ui.selectable_value(
                                    &mut app.rule_draft.action,
                                    index,
                                    action_label(lang, index),
                                );
                            }
                        });
                    if action_needs_folder(app.rule_draft.action) {
                        widgets::text_input(
                            ui,
                            palette,
                            &mut app.rule_draft.target,
                            i18n::t(lang, Key::RuleTarget),
                            220.0,
                        );
                    }
                });
                if action_needs_template(app.rule_draft.action) {
                    widgets::text_input(
                        ui,
                        palette,
                        &mut app.rule_draft.template,
                        i18n::t(lang, Key::RuleTemplate),
                        260.0,
                    );
                    widgets::small_muted(ui, palette, i18n::t(lang, Key::RuleTemplateHint));
                }
                ui.checkbox(
                    &mut app.rule_draft.sub_enabled,
                    i18n::t(lang, Key::RuleSubfolderLabel),
                );
                if app.rule_draft.sub_enabled {
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut app.rule_draft.sub_year_month,
                            i18n::t(lang, Key::SubByDate),
                        );
                        ui.checkbox(&mut app.rule_draft.sub_type, i18n::t(lang, Key::SubByType));
                    });
                }
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let save = if app.rule_draft.editing.is_some() {
                Key::RuleUpdate
            } else {
                Key::RuleSave
            };
            if widgets::accent_button(ui, palette, i18n::t(lang, save)).clicked() {
                save_simple_rule(app, scope, rules, changed);
            }
            if widgets::ghost_button(ui, palette, i18n::t(lang, Key::RuleCancel)).clicked() {
                app.rule_draft = RuleDraft::default();
                app.rule_error = None;
            }
        });
    });
}

fn save_simple_rule(app: &mut App, scope: Scope, rules: &mut Vec<Rule>, changed: &mut bool) {
    let draft = &mut app.rule_draft;
    if draft.target.trim().is_empty() {
        app.rule_error = Some(i18n::t(app.lang, Key::RuleDestinationRequired).to_string());
        return;
    }
    if draft.simple_value.trim().is_empty() {
        app.rule_error = Some(i18n::t(app.lang, Key::RuleConditionValueRequired).to_string());
        return;
    }
    if matches!(draft.simple_kind, 5 | 7)
        && draft
            .simple_value
            .trim()
            .parse::<u64>()
            .map(|value| value == 0)
            .unwrap_or(true)
    {
        app.rule_error = Some(i18n::t(app.lang, Key::RuleConditionNumber).to_string());
        return;
    }

    if draft.name.trim().is_empty() {
        let target = if draft.target.trim().is_empty() {
            i18n::t(app.lang, Key::RuleDestination)
        } else {
            draft.target.trim()
        };
        let value = draft.simple_value.trim();
        draft.name = if value.is_empty() {
            target.to_string()
        } else {
            format!("{value} → {target}")
        };
    }

    if !draft.simple_value.trim().is_empty() {
        let simple = match draft.simple_kind {
            0 => {
                let value = config::normalize_extension(&draft.simple_value);
                Condition::Extension { value }
            }
            5 => Condition::MinSize {
                value: draft
                    .simple_value
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(0)
                    .saturating_mul(1_048_576),
            },
            7 => Condition::OlderThan {
                value: draft.simple_value.trim().parse().unwrap_or(0),
            },
            _ => Condition::NameContains {
                value: draft.simple_value.trim().to_string(),
            },
        };
        if draft.conditions.is_empty() {
            draft.conditions.push(simple);
        } else {
            draft.conditions[0] = simple;
        }
    }

    let next_id = rules.iter().map(|rule| rule.id).max().unwrap_or(0) + 1;
    let date_source = app.scope_settings(scope).subfolders.date_source;
    match build_rule(app, next_id, date_source) {
        Ok(rule) => {
            if let Some(id) = app.rule_draft.editing {
                if let Some(existing) = rules.iter_mut().find(|item| item.id == id) {
                    *existing = Rule { id, ..rule };
                } else {
                    rules.push(rule);
                }
            } else {
                rules.push(rule);
            }
            app.rule_draft = RuleDraft::default();
            app.rule_error = None;
            *changed = true;
        }
        Err(message) => app.rule_error = Some(message),
    }
}

fn simple_kind_label(lang: i18n::Lang, index: usize) -> String {
    let key = match index {
        0 => Key::RuleTypeIs,
        5 => Key::RuleWeighsMore,
        7 => Key::RuleOlderThan,
        _ => Key::RuleNameContains,
    };
    i18n::t(lang, key).to_string()
}

fn live_preview(app: &mut App, ui: &mut egui::Ui, palette: &Palette, scope: Scope) {
    let lang = app.lang;
    widgets::card_titled(ui, palette, i18n::t(lang, Key::LivePreviewTitle), |ui| {
        widgets::small_muted(ui, palette, i18n::t(lang, Key::LivePreviewHint));
        ui.add_space(6.0);
        widgets::text_input(
            ui,
            palette,
            &mut app.live_preview,
            i18n::t(lang, Key::RuleTesterName),
            260.0,
        );
        ui.add_space(6.0);
        let name = app.live_preview.trim().to_string();
        if name.is_empty() {
            return;
        }
        let settings = app.scope_settings(scope);
        let target = match scope {
            Scope::Root(index) => app
                .settings
                .roots
                .get(index)
                .map(|root| rules::preview_destination_in(&settings, &root.path, &name))
                .unwrap_or_else(|| rules::preview_destination(&settings, &name)),
            Scope::Global => rules::preview_destination(&settings, &name),
        };
        match target {
            Target::Move { dest, .. } | Target::Copy { dest, .. } => {
                ui.label(
                    RichText::new(
                        i18n::t(lang, Key::LivePreviewGoes)
                            .replace("{name}", &name)
                            .replace("{destination}", &dest.display().to_string()),
                    )
                    .size(13.0)
                    .color(palette.text),
                );
            }
            Target::Trash { .. } => {
                ui.label(
                    RichText::new(i18n::t(lang, Key::LivePreviewTrash).replace("{name}", &name))
                        .size(13.0)
                        .color(palette.text),
                );
            }
            Target::Leave(_) => {
                widgets::small_muted(
                    ui,
                    palette,
                    &i18n::t(lang, Key::LivePreviewStays).replace("{name}", &name),
                );
            }
        }
    });
}

fn rule_folder(action: &Action) -> Option<&str> {
    match action {
        Action::MoveTo { value } | Action::CopyTo { value } => Some(value),
        Action::MoveRename { folder, .. } | Action::CopyRename { folder, .. } => Some(folder),
        Action::Trash | Action::Rename { .. } | Action::Ignore => None,
    }
}

fn condition_value_input(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    let lang = app.lang;
    let kind = app.rule_draft.condition_kind;

    if kind == 11 {
        if app.rule_draft.condition_value.is_empty() {
            app.rule_draft.condition_value = "image".to_string();
        }
        egui::ComboBox::from_id_salt("regla-tipo-real")
            .selected_text(real_type_label(lang, &app.rule_draft.condition_value))
            .width(140.0)
            .show_ui(ui, |ui| {
                for (value, _) in real_types() {
                    ui.selectable_value(
                        &mut app.rule_draft.condition_value,
                        value.to_string(),
                        real_type_label(lang, value),
                    );
                }
            });
        return;
    }

    if kind == 13 && !cfg!(feature = "regex-rules") {
        ui.label(
            RichText::new(i18n::t(lang, Key::RegexUnavailableCondition))
                .size(13.0)
                .color(palette.error),
        );
    }

    let hint = match kind {
        0 => i18n::t(lang, Key::ExtensionPlaceholder),
        1 => "*.pdf",
        12 => i18n::t(lang, Key::SourceFolderHint),
        _ => "",
    };
    widgets::text_input(
        ui,
        palette,
        &mut app.rule_draft.condition_value,
        hint,
        160.0,
    );

    if kind == 5 || kind == 6 {
        egui::ComboBox::from_id_salt("regla-unidad")
            .selected_text(["KB", "MB", "GB"][app.rule_draft.condition_unit.min(2)])
            .width(70.0)
            .show_ui(ui, |ui| {
                for (index, unit) in ["KB", "MB", "GB"].iter().enumerate() {
                    ui.selectable_value(&mut app.rule_draft.condition_unit, index, *unit);
                }
            });
    } else if (7..=10).contains(&kind) {
        ui.label(
            RichText::new(i18n::t(lang, Key::ConditionDays))
                .size(13.0)
                .color(palette.muted),
        );
    }
}

fn tester(app: &mut App, ui: &mut egui::Ui, palette: &Palette, scope: Scope) {
    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::RuleTester), |ui| {
        let lang = app.lang;
        ui.horizontal_wrapped(|ui| {
            widgets::text_input(
                ui,
                palette,
                &mut app.tester.name,
                i18n::t(lang, Key::RuleTesterName),
                220.0,
            );
            widgets::text_input(
                ui,
                palette,
                &mut app.tester.size_kb,
                i18n::t(lang, Key::RuleTesterSize),
                90.0,
            );
            widgets::text_input(
                ui,
                palette,
                &mut app.tester.days,
                i18n::t(lang, Key::RuleTesterDays),
                90.0,
            );
        });

        ui.add_space(8.0);

        let name = app.tester.name.trim().to_string();
        if name.is_empty() {
            widgets::small_muted(ui, palette, i18n::t(lang, Key::RuleTesterName));
            return;
        }

        let size_kb: u64 = app.tester.size_kb.trim().parse().unwrap_or(0);
        let days: u32 = app.tester.days.trim().parse().unwrap_or(0);
        let settings = app.scope_settings(scope);
        let root = app.scope_root(scope);
        let result = rules::test(&settings, root, &name, size_kb.saturating_mul(1024), days);

        if let Some(source) = &result.source {
            ui.label(
                RichText::new(format!("{}: {source}", i18n::t(lang, Key::RuleTesterMatch)))
                    .size(13.0)
                    .color(palette.text),
            );
        } else {
            widgets::small_muted(ui, palette, i18n::t(lang, Key::RuleTesterNoMatch));
        }

        let target = match &result.target {
            Target::Move { dest, name, .. } | Target::Copy { dest, name, .. } => {
                let copy = matches!(&result.target, Target::Copy { .. });
                let mut text = dest.join(name).display().to_string();
                if copy {
                    text.push_str("  [");
                    text.push_str(i18n::t(lang, Key::ActionCopy));
                    text.push(']');
                }
                Some(text)
            }
            Target::Trash { .. } => Some(i18n::t(lang, Key::Trash).to_string()),
            Target::Leave(_) => {
                widgets::small_muted(ui, palette, i18n::t(lang, Key::RuleTesterLeave));
                None
            }
        };

        if let Some(target) = target {
            ui.label(
                RichText::new(target)
                    .size(13.0)
                    .monospace()
                    .color(palette.accent_text),
            );
        }

        let has_real_type = settings.rules.iter().any(|rule| {
            rule.conditions
                .iter()
                .any(|condition| matches!(condition, Condition::RealType { .. }))
        });
        if has_real_type {
            widgets::small_muted(ui, palette, i18n::t(lang, Key::RuleTesterRealTypeNote));
        }
        if !cfg!(feature = "regex-rules")
            && settings
                .rules
                .iter()
                .filter(|rule| rule.enabled)
                .any(|rule| {
                    rule.conditions
                        .iter()
                        .any(|condition| matches!(condition, Condition::Regex { .. }))
                })
        {
            ui.label(
                RichText::new(i18n::t(lang, Key::RegexUnavailable))
                    .size(13.0)
                    .color(palette.error),
            );
        }
    });
}

fn build_rule(app: &App, next_id: u64, date_source: DateSource) -> Result<Rule, String> {
    let draft = &app.rule_draft;
    let lang = app.lang;

    if draft.name.trim().is_empty() {
        return Err(i18n::t(lang, Key::RuleNameRequired).to_string());
    }
    if draft.conditions.is_empty() {
        return Err(i18n::t(lang, Key::RuleConditionRequired).to_string());
    }

    let folder = draft.target.trim().to_string();
    let template = draft.template.trim().to_string();

    let action = match draft.action {
        0 => {
            if folder.is_empty() {
                return Err(i18n::t(lang, Key::RuleConditionValueRequired).to_string());
            }
            Action::MoveTo { value: folder }
        }
        1 => Action::Trash,
        2 => {
            return Err(i18n::t(lang, Key::ActionRenameUnavailable).to_string());
        }
        3 => {
            if folder.is_empty() {
                return Err(i18n::t(lang, Key::RuleConditionValueRequired).to_string());
            }
            Action::CopyTo { value: folder }
        }
        4 => {
            if folder.is_empty() || template.is_empty() {
                return Err(i18n::t(lang, Key::RuleConditionValueRequired).to_string());
            }
            Action::MoveRename { folder, template }
        }
        5 => {
            if folder.is_empty() || template.is_empty() {
                return Err(i18n::t(lang, Key::RuleConditionValueRequired).to_string());
            }
            Action::CopyRename { folder, template }
        }
        _ => Action::Ignore,
    };

    let enabled = draft.enabled;

    Ok(Rule {
        id: draft.editing.unwrap_or(next_id),
        name: draft.name.trim().to_string(),
        enabled,
        conditions: draft.conditions.clone(),
        match_all: draft.match_all,
        action,
        subfolder: RuleSubfolder {
            enabled: draft.sub_enabled,
            by_year_month: draft.sub_enabled && draft.sub_year_month,
            by_type: draft.sub_enabled && draft.sub_type,
            date_source,
        },
        legacy_matcher: None,
    })
}

fn build_condition(app: &App) -> Result<Condition, String> {
    let draft = &app.rule_draft;
    let lang = app.lang;
    let text = draft.condition_value.trim();
    let required = || i18n::t(lang, Key::RuleConditionValueRequired).to_string();

    let condition = match draft.condition_kind {
        0 => {
            let value = crate::config::normalize_extension(text);
            if value.is_empty() {
                return Err(required());
            }
            Condition::Extension { value }
        }
        1 => {
            if text.is_empty() {
                return Err(required());
            }
            Condition::Glob {
                value: text.to_string(),
            }
        }
        2 => {
            if text.is_empty() {
                return Err(required());
            }
            Condition::NameContains {
                value: text.to_string(),
            }
        }
        3 => {
            if text.is_empty() {
                return Err(required());
            }
            Condition::NameStartsWith {
                value: text.to_string(),
            }
        }
        4 => {
            if text.is_empty() {
                return Err(required());
            }
            Condition::NameEndsWith {
                value: text.to_string(),
            }
        }
        5 | 6 => {
            let number: u64 = text.parse().map_err(|_| required())?;
            let factor = [1024u64, 1_048_576, 1_073_741_824][draft.condition_unit.min(2)];
            let bytes = number.saturating_mul(factor);
            if draft.condition_kind == 5 {
                Condition::MinSize { value: bytes }
            } else {
                Condition::MaxSize { value: bytes }
            }
        }
        7..=10 => {
            let days: u32 = text.parse().map_err(|_| required())?;
            match draft.condition_kind {
                7 => Condition::OlderThan { value: days },
                8 => Condition::NewerThan { value: days },
                9 => Condition::CreatedOlderThan { value: days },
                _ => Condition::CreatedNewerThan { value: days },
            }
        }
        11 => Condition::RealType {
            value: if text.is_empty() {
                "image".to_string()
            } else {
                text.to_string()
            },
        },
        12 => {
            if text.is_empty() {
                return Err(required());
            }
            Condition::SourceFolder {
                value: text.to_string(),
            }
        }
        _ => {
            if !cfg!(feature = "regex-rules") {
                return Err(i18n::t(lang, Key::RegexUnavailable).to_string());
            }
            if text.is_empty() {
                return Err(required());
            }
            #[cfg(feature = "regex-rules")]
            if regex_lite::Regex::new(text).is_err() {
                return Err(i18n::t(lang, Key::RegexInvalid).to_string());
            }
            Condition::Regex {
                value: text.to_string(),
            }
        }
    };

    Ok(condition)
}

fn action_needs_folder(index: usize) -> bool {
    matches!(index, 0 | 2 | 3 | 4 | 5)
}

fn action_needs_template(index: usize) -> bool {
    matches!(index, 4 | 5)
}

pub fn condition_text(lang: i18n::Lang, condition: &Condition) -> String {
    match condition {
        Condition::Extension { value } => {
            format!(
                "{} {}",
                i18n::t(lang, Key::KindExtension),
                i18n::file_type_name(lang, value)
            )
        }
        Condition::Glob { value } => format!("{} {value}", i18n::t(lang, Key::KindGlob)),
        Condition::NameContains { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindNameContains))
        }
        Condition::NameStartsWith { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindNameStarts))
        }
        Condition::NameEndsWith { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindNameEnds))
        }
        Condition::MinSize { value } => format!(
            "{} {}",
            i18n::t(lang, Key::KindMinSize),
            i18n::bytes_size(lang, *value)
        ),
        Condition::MaxSize { value } => format!(
            "{} {}",
            i18n::t(lang, Key::KindMaxSize),
            i18n::bytes_size(lang, *value)
        ),
        Condition::OlderThan { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindOlder))
        }
        Condition::NewerThan { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindNewer))
        }
        Condition::CreatedOlderThan { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindCreatedOlder))
        }
        Condition::CreatedNewerThan { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindCreatedNewer))
        }
        Condition::RealType { value } => {
            format!(
                "{}: {}",
                i18n::t(lang, Key::KindRealType),
                real_type_label(lang, value)
            )
        }
        Condition::SourceFolder { value } => {
            format!("{}: {value}", i18n::t(lang, Key::KindSourceFolder))
        }
        Condition::Regex { value } => {
            format!("{} {value}", i18n::t(lang, Key::KindRegex))
        }
    }
}

pub fn action_text(lang: i18n::Lang, rule: &Rule) -> String {
    match &rule.action {
        Action::MoveTo { value } => format!("{} {value}", i18n::t(lang, Key::ActionMove)),
        Action::Trash => i18n::t(lang, Key::ActionTrash).to_string(),
        Action::Rename { value } => format!("{} {value}", i18n::t(lang, Key::ActionRename)),
        Action::CopyTo { value } => format!("{} {value}", i18n::t(lang, Key::ActionCopy)),
        Action::MoveRename { folder, template } => format!(
            "{} {folder} ({template})",
            i18n::t(lang, Key::ActionMoveRename)
        ),
        Action::CopyRename { folder, template } => format!(
            "{} {folder} ({template})",
            i18n::t(lang, Key::ActionCopyRename)
        ),
        Action::Ignore => i18n::t(lang, Key::ActionIgnore).to_string(),
    }
}

fn kind_label(lang: i18n::Lang, index: usize) -> String {
    let key = match index {
        0 => Key::KindExtension,
        1 => Key::KindGlob,
        2 => Key::KindNameContains,
        3 => Key::KindNameStarts,
        4 => Key::KindNameEnds,
        5 => Key::KindMinSize,
        6 => Key::KindMaxSize,
        7 => Key::KindOlder,
        8 => Key::KindNewer,
        9 => Key::KindCreatedOlder,
        10 => Key::KindCreatedNewer,
        11 => Key::KindRealType,
        12 => Key::KindSourceFolder,
        _ => Key::KindRegex,
    };
    i18n::t(lang, key).to_string()
}

fn action_label(lang: i18n::Lang, index: usize) -> String {
    let key = match index {
        0 => Key::ActionMove,
        1 => Key::ActionTrash,
        2 => Key::ActionRename,
        3 => Key::ActionCopy,
        4 => Key::ActionMoveRename,
        5 => Key::ActionCopyRename,
        _ => Key::ActionIgnore,
    };
    i18n::t(lang, key).to_string()
}

fn real_types() -> [(&'static str, Key); 9] {
    [
        ("image", Key::ConditionRealImage),
        ("video", Key::ConditionRealVideo),
        ("audio", Key::ConditionRealAudio),
        ("document", Key::ConditionRealDocument),
        ("archive", Key::ConditionRealArchive),
        ("text", Key::ConditionRealText),
        ("font", Key::ConditionRealFont),
        ("program", Key::ConditionRealProgram),
        ("book", Key::ConditionRealBook),
    ]
}

fn real_type_label(lang: i18n::Lang, value: &str) -> String {
    real_types()
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, key)| i18n::t(lang, *key).to_string())
        .unwrap_or_else(|| value.to_string())
}
