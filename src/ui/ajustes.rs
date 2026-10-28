//! Ajustes: general, carpetas, reglas, vigilancia, apariencia y avanzado.

use eframe::egui::{self, Align, Layout, RichText};

use super::reglas;
use super::theme::{self, ACCENTS, Palette};
use super::widgets;
use super::{App, Confirm, Notice, Scope, SettingsSection, Tab, data_folder};
use crate::config::{self, DateSource, Density, FolderProblem, Profile, TextSize, Theme};
use crate::i18n::{self, Key, Lang};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::palette(ui, &app.settings);
    let mut dirty = false;

    if let Scope::Root(index) = app.scope {
        if index >= app.settings.roots.len() || app.scope_categories(app.scope).is_none() {
            app.scope = Scope::Global;
        }
    }

    egui::Panel::left("ajustes_indice")
        .exact_size(188.0)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let mut sections = vec![
                    (SettingsSection::General, Key::GeneralTitle),
                    (SettingsSection::Carpetas, Key::FoldersTitle),
                    (SettingsSection::Estilo, Key::SectionStyle),
                    (SettingsSection::Apariencia, Key::AppearanceTitle),
                ];
                if app.settings.advanced {
                    sections.insert(3, (SettingsSection::Reglas, Key::RulesTitle));
                    sections.insert(4, (SettingsSection::Vigilancia, Key::WatchTitle));
                    sections.insert(5, (SettingsSection::Duplicados, Key::DuplicatesTitle));
                    sections.push((SettingsSection::Avanzado, Key::AdvancedTitle));
                }
                for (section, key) in sections {
                    if widgets::nav_entry(
                        ui,
                        &palette,
                        i18n::t(app.lang, key),
                        app.section == section,
                    )
                    .clicked()
                    {
                        app.section = section;
                    }
                    ui.add_space(2.0);
                }
            });
        });

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            match app.section {
                SettingsSection::General => general(app, ui, &palette, &mut dirty),
                SettingsSection::Carpetas => folders(app, ui, &palette, &mut dirty),
                SettingsSection::Estilo => style(app, ui, &palette, &mut dirty),
                SettingsSection::Reglas => rules(app, ui, &palette, &mut dirty),
                SettingsSection::Vigilancia => watch(app, ui, &palette, &mut dirty),
                SettingsSection::Duplicados => super::duplicados::show(app, ui),
                SettingsSection::Apariencia => appearance(app, ui, &palette, &mut dirty),
                SettingsSection::Avanzado => advanced(app, ui, &palette, &mut dirty),
            }
            ui.add_space(12.0);
        });

    folder_edit_modal(app, ui.ctx(), &palette);

    if dirty {
        app.save();
    }
}

fn header(ui: &mut egui::Ui, palette: &Palette, title: &str, hint: &str) {
    ui.label(RichText::new(title).size(22.0).strong().color(palette.text));
    if !hint.is_empty() {
        ui.add_space(2.0);
        widgets::small_muted(ui, palette, hint);
    }
    ui.add_space(14.0);
}

fn general(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(ui, palette, i18n::t(app.lang, Key::GeneralTitle), "");

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ModeTitle), |ui| {
        let before = app.settings.advanced;
        let options = [
            i18n::t(app.lang, Key::ModeSimple),
            i18n::t(app.lang, Key::ModeAdvanced),
        ];
        if let Some(picked) =
            widgets::segmented(ui, palette, &options, usize::from(app.settings.advanced))
        {
            app.settings.advanced = picked == 1;
        }
        if app.settings.advanced != before {
            *dirty = true;
            if !app.settings.advanced {
                if app.tab == Tab::Organizar {
                    app.tab = Tab::Ajustes;
                }
                if matches!(
                    app.section,
                    SettingsSection::Reglas
                        | SettingsSection::Vigilancia
                        | SettingsSection::Duplicados
                        | SettingsSection::Avanzado
                ) {
                    app.section = SettingsSection::General;
                }
            }
        }
        ui.add_space(6.0);
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::ModeHint));
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::LanguageLabel), |ui| {
        let before = app.lang;
        let options = [
            i18n::t(app.lang, Key::LanguageSpanish),
            i18n::t(Lang::En, Key::LanguageEnglish),
        ];
        if let Some(picked) =
            widgets::segmented(ui, palette, &options, usize::from(app.lang == Lang::En))
        {
            app.lang = if picked == 1 { Lang::En } else { Lang::Es };
        }
        if app.lang != before {
            app.settings.language = app.lang.code().to_string();
            *dirty = true;
        }
    });

    ui.add_space(12.0);

    widgets::card_titled(
        ui,
        palette,
        i18n::t(app.lang, Key::NotificationsTitle),
        |ui| {
            if widgets::switch(
                ui,
                palette,
                &mut app.settings.silent,
                i18n::t(app.lang, Key::SilentMode),
                app.lang,
            )
            .changed()
            {
                *dirty = true;
            }
            ui.add_space(4.0);
            widgets::small_muted(ui, palette, i18n::t(app.lang, Key::SilentHint));
        },
    );

    ui.add_space(12.0);

    profile_section(app, ui, palette, dirty);
}

fn profile_section(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    let mut apply = None;
    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ProfileTitle), |ui| {
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::ProfileOnlyMissing));
        ui.add_space(8.0);

        for profile in Profile::ALL {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width((ui.available_width() - 160.0).max(180.0));
                    ui.label(
                        RichText::new(profile_label(app.lang, profile))
                            .size(14.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.label(
                        RichText::new(profile_description(app.lang, profile))
                            .size(13.0)
                            .color(palette.muted),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let label = format!(
                        "{}: {}",
                        i18n::t(app.lang, Key::ViewChanges),
                        profile_label(app.lang, profile)
                    );
                    if widgets::ghost_button(ui, palette, &label).clicked() {
                        app.profile_choice = Some(profile);
                    }
                });
            });
            ui.add_space(4.0);
        }

        let Some(profile) = app.profile_choice else {
            return;
        };

        ui.add_space(8.0);
        widgets::divider(ui, palette);
        ui.add_space(6.0);

        let mut preview_settings = app.settings.clone();
        let (groups, rules, skipped) = preview_settings.apply_profile(profile);
        let added_groups: Vec<String> = preview_settings
            .categories
            .iter()
            .skip(app.settings.categories.len())
            .map(|category| category.name.clone())
            .collect();
        let added_rules: Vec<String> = preview_settings
            .rules
            .iter()
            .skip(app.settings.rules.len())
            .map(|rule| rule.name.clone())
            .collect();

        ui.label(
            RichText::new(i18n::profile_changes(app.lang, groups, rules, skipped))
                .size(13.0)
                .color(palette.text),
        );
        if !added_groups.is_empty() {
            ui.label(
                RichText::new(format!(
                    "{}: {}",
                    i18n::t(app.lang, Key::FoldersTitle),
                    added_groups.join(", ")
                ))
                .size(13.0)
                .color(palette.muted),
            );
        }
        if !added_rules.is_empty() {
            ui.label(
                RichText::new(format!(
                    "{}: {}",
                    i18n::t(app.lang, Key::RulesTitle),
                    added_rules.join(", ")
                ))
                .size(13.0)
                .color(palette.muted),
            );
        }

        ui.add_space(8.0);
        if widgets::accent_button_enabled(
            ui,
            palette,
            i18n::t(app.lang, Key::ProfileApply),
            groups + rules > 0,
        )
        .clicked()
        {
            apply = Some(profile);
        }
    });

    if let Some(profile) = apply {
        let previous = Box::new(app.settings.clone());
        let (groups, rules, skipped) = app.settings.apply_profile(profile);
        app.undo_change = Some(super::UndoChange::Settings { previous });
        app.profile_choice = None;
        *dirty = true;
        app.notice = Some(Notice::info(i18n::profile_changes(
            app.lang, groups, rules, skipped,
        )));
    }
}

fn profile_description(lang: Lang, profile: Profile) -> &'static str {
    match profile {
        Profile::General => i18n::t(lang, Key::ProfileDescriptionBasic),
        Profile::Student => i18n::t(lang, Key::ProfileDescriptionStudent),
        Profile::Developer => i18n::t(lang, Key::ProfileDescriptionDeveloper),
        Profile::Photographer => i18n::t(lang, Key::ProfileDescriptionPhotographer),
    }
}

fn folders(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(ui, palette, i18n::t(app.lang, Key::FoldersTitle), "");

    if app.settings.roots.is_empty() {
        widgets::card(ui, palette, |ui| {
            widgets::small_muted(ui, palette, i18n::t(app.lang, Key::FolderEmptyShortcuts));
            ui.add_space(8.0);
            if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::AddFolder)).clicked() {
                app.pick_folder();
            }
            ui.add_space(8.0);
            app.shortcut_buttons(ui, palette);
        });
        return;
    }

    let mut roots = app.settings.roots.clone();
    let mut remove: Option<usize> = None;
    let mut roots_changed = false;

    for (index, root) in roots.iter_mut().enumerate() {
        if app.root_exclude_files.len() <= index {
            app.root_exclude_files
                .push(root.overrides.exclude_files.join(", "));
        }
        if app.root_exclude_dirs.len() <= index {
            app.root_exclude_dirs
                .push(root.overrides.exclude_dirs.join(", "));
        }

        widgets::card(ui, palette, |ui| {
            ui.horizontal(|ui| {
                widgets::folder_icon(ui, palette);
                ui.vertical(|ui| {
                    ui.set_max_width((ui.available_width() - 190.0).max(120.0));
                    let name = root
                        .path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| root.path.display().to_string());
                    ui.add(
                        egui::Label::new(
                            RichText::new(name).size(15.0).strong().color(palette.text),
                        )
                        .truncate(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(root.path.display().to_string())
                                .size(13.0)
                                .color(palette.muted),
                        )
                        .truncate(),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::danger_button(ui, palette, i18n::t(app.lang, Key::Remove)).clicked()
                    {
                        remove = Some(index);
                    }
                    if widgets::link_button(ui, palette, i18n::t(app.lang, Key::OpenFolder))
                        .clicked()
                    {
                        app.open_path(&root.path);
                    }
                    if !root.path.exists() {
                        widgets::pill(
                            ui,
                            palette,
                            i18n::t(app.lang, Key::FolderMissing),
                            palette.error,
                        );
                    }
                });
            });

            ui.add_space(4.0);
            let label = if root.enabled {
                i18n::t(app.lang, Key::FolderActive)
            } else {
                i18n::t(app.lang, Key::FolderPaused)
            };
            if ui.checkbox(&mut root.enabled, label).changed() {
                roots_changed = true;
            }

            egui::CollapsingHeader::new(i18n::t(app.lang, Key::RootOverrides))
                .id_salt(("root-overrides", index))
                .show(ui, |ui| {
                    let mut own = root.overrides.subfolders.is_some();
                    if ui
                        .checkbox(&mut own, i18n::t(app.lang, Key::RootOwnSubfolders))
                        .changed()
                    {
                        root.overrides.subfolders = if own {
                            Some(app.settings.subfolders.clone())
                        } else {
                            None
                        };
                        roots_changed = true;
                    }

                    if let Some(subfolders) = &mut root.overrides.subfolders {
                        subfolder_controls(app, ui, subfolders, index, &mut roots_changed);
                    }

                    ui.add_space(6.0);

                    let files_changed = widgets::text_input(
                        ui,
                        palette,
                        &mut app.root_exclude_files[index],
                        i18n::t(app.lang, Key::RootExcludeFiles),
                        360.0,
                    )
                    .changed();
                    if files_changed {
                        root.overrides.exclude_files = split_list(&app.root_exclude_files[index]);
                        roots_changed = true;
                    }

                    let dirs_changed = widgets::text_input(
                        ui,
                        palette,
                        &mut app.root_exclude_dirs[index],
                        i18n::t(app.lang, Key::RootExcludeDirs),
                        360.0,
                    )
                    .changed();
                    if dirs_changed {
                        root.overrides.exclude_dirs = split_list(&app.root_exclude_dirs[index]);
                        roots_changed = true;
                    }

                    ui.add_space(8.0);

                    let mut own_rules = root.overrides.rules.is_some();
                    if ui
                        .checkbox(&mut own_rules, i18n::t(app.lang, Key::RootOwnRules))
                        .changed()
                    {
                        if own_rules {
                            root.overrides.categories = Some(app.settings.categories.clone());
                            root.overrides.rules = Some(app.settings.rules.clone());
                        } else {
                            root.overrides.categories = None;
                            root.overrides.rules = None;
                            if app.scope == Scope::Root(index) {
                                app.scope = Scope::Global;
                            }
                        }
                        roots_changed = true;
                    }
                    widgets::small_muted(ui, palette, i18n::t(app.lang, Key::RootOwnHint));

                    if own_rules
                        && widgets::link_button(ui, palette, i18n::t(app.lang, Key::RootEditRules))
                            .clicked()
                    {
                        app.scope = Scope::Root(index);
                        app.section = SettingsSection::Reglas;
                    }
                });
        });

        ui.add_space(8.0);
    }

    if roots_changed {
        app.settings.roots = roots;
        *dirty = true;
        app.plan = None;
    }

    ui.add_space(4.0);
    if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::AddFolder)).clicked() {
        app.pick_folder();
    }
    ui.add_space(8.0);
    app.shortcut_buttons(ui, palette);

    if let Some(index) = remove {
        app.remove_root(index);
    }
}

fn rules(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(
        ui,
        palette,
        i18n::t(app.lang, Key::RuleSpecialTitle),
        i18n::t(app.lang, Key::RuleSpecialHint),
    );

    scope_combo(app, ui, palette);

    ui.add_space(12.0);

    let scope = app.scope;
    reglas::editor(app, ui, palette, scope, dirty);
}

fn style(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(
        ui,
        palette,
        i18n::t(app.lang, Key::StyleTitle),
        i18n::t(app.lang, Key::StyleHint),
    );

    ui.horizontal(|ui| {
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::SubfoldersLabel));
        widgets::help_icon(ui, palette, i18n::t(app.lang, Key::TooltipSubfolders));
    });

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::StyleTitle), |ui| {
        let mut by_date = app.settings.subfolders.by_year_month;
        let mut changed = false;
        if ui
            .radio_value(&mut by_date, false, i18n::t(app.lang, Key::StyleByType))
            .changed()
        {
            changed = true;
        }
        if ui
            .radio_value(
                &mut by_date,
                true,
                i18n::t(app.lang, Key::StyleByTypeAndDate),
            )
            .changed()
        {
            changed = true;
        }
        if changed {
            app.settings.subfolders.by_capture = false;
            *dirty = true;
        }
        if !app.settings.subfolders.by_type {
            changed = true;
        }
        app.settings.subfolders.by_type = true;
        app.settings.subfolders.by_year_month = by_date;
        if changed {
            *dirty = true;
        }
    });
}

fn scope_combo(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    let mut choices: Vec<(Scope, String)> = vec![(
        Scope::Global,
        i18n::t(app.lang, Key::ScopeGlobal).to_string(),
    )];

    for (index, root) in app.settings.roots.iter().enumerate() {
        if root.overrides.categories.is_some() || root.overrides.rules.is_some() {
            let name = root
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.path.display().to_string());
            choices.push((Scope::Root(index), name));
        }
    }

    if !choices.iter().any(|(scope, _)| *scope == app.scope) {
        app.scope = Scope::Global;
    }

    ui.horizontal(|ui| {
        ui.add_sized(
            egui::vec2(130.0, 28.0),
            egui::Label::new(
                RichText::new(i18n::t(app.lang, Key::ScopeLabel))
                    .size(13.0)
                    .color(palette.muted),
            ),
        );
        let selected = choices
            .iter()
            .find(|(scope, _)| *scope == app.scope)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| i18n::t(app.lang, Key::ScopeGlobal).to_string());

        let before = app.scope;
        egui::ComboBox::from_id_salt("ajustes-ambito")
            .selected_text(selected)
            .width(260.0)
            .show_ui(ui, |ui| {
                for (scope, name) in &choices {
                    ui.selectable_value(&mut app.scope, *scope, name.as_str());
                }
            });
        if app.scope != before {
            app.rule_draft = super::RuleDraft::default();
            app.draft_type.clear();
            app.expanded_group = None;
        }
        widgets::help_icon(ui, palette, i18n::t(app.lang, Key::TooltipScope));
    });
}

fn watch(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(
        ui,
        palette,
        i18n::t(app.lang, Key::WatchTitle),
        i18n::t(app.lang, Key::WatchHint),
    );

    ui.horizontal(|ui| {
        widgets::help_icon(ui, palette, i18n::t(app.lang, Key::TooltipWatch));
    });

    let running = app.watcher_running();

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::WatchTitle), |ui| {
        ui.horizontal(|ui| {
            widgets::pill(
                ui,
                palette,
                if running {
                    i18n::t(app.lang, Key::WatchOn)
                } else {
                    i18n::t(app.lang, Key::WatchOff)
                },
                if running {
                    palette.success
                } else {
                    palette.muted
                },
            );

            if running {
                if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::RestartWatch))
                    .clicked()
                {
                    app.restart_watcher();
                }
                if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::StopWatch)).clicked() {
                    app.stop_watcher();
                }
            } else if (app.watch_failed || app.watch_retry_at.is_some())
                && widgets::accent_button(ui, palette, i18n::t(app.lang, Key::RestartWatch))
                    .clicked()
            {
                app.restart_watcher();
            } else if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::StartWatch))
                .clicked()
            {
                app.start_watcher();
            }
        });

        if app.watch_failed {
            ui.add_space(8.0);
            ui.label(
                RichText::new(i18n::t(app.lang, Key::WatchFailed))
                    .size(13.0)
                    .color(palette.error),
            );
        }

        ui.add_space(8.0);

        if ui
            .checkbox(
                &mut app.settings.background,
                i18n::t(app.lang, Key::BackgroundLabel),
            )
            .changed()
        {
            *dirty = true;
        }
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::BackgroundHint));

        ui.add_space(4.0);
        let mut autostart = app.settings.start_with_windows;
        if ui
            .checkbox(&mut autostart, i18n::t(app.lang, Key::AutostartLabel))
            .changed()
        {
            match config::set_autostart(autostart) {
                Ok(()) => {
                    app.settings.start_with_windows = autostart;
                    *dirty = true;
                }
                Err(err) => {
                    app.notice = Some(Notice::error(err.describe(app.lang)));
                }
            }
        }
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::AutostartHint));
        ui.add_space(4.0);
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::WatchRestartHint));
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ScheduleLabel), |ui| {
        ui.horizontal(|ui| {
            ui.add_sized(
                egui::vec2(190.0, 20.0),
                egui::Label::new(
                    RichText::new(i18n::t(app.lang, Key::ScheduleLabel))
                        .size(13.0)
                        .color(palette.muted),
                ),
            );
            if ui
                .add(
                    egui::DragValue::new(&mut app.settings.watch.schedule_minutes)
                        .range(0..=1440)
                        .speed(1.0),
                )
                .changed()
            {
                *dirty = true;
            }
        });
        if app.settings.watch.schedule_minutes == 0 {
            widgets::small_muted(ui, palette, i18n::t(app.lang, Key::ScheduleOff));
        }
        ui.horizontal(|ui| {
            ui.add_sized(
                egui::vec2(190.0, 20.0),
                egui::Label::new(
                    RichText::new(i18n::t(app.lang, Key::DebounceLabel))
                        .size(13.0)
                        .color(palette.muted),
                ),
            );
            if ui
                .add(
                    egui::DragValue::new(&mut app.settings.watch.debounce_ms)
                        .range(200..=30_000)
                        .speed(50.0),
                )
                .changed()
            {
                *dirty = true;
            }
        });
        ui.horizontal(|ui| {
            ui.add_sized(
                egui::vec2(190.0, 20.0),
                egui::Label::new(
                    RichText::new(i18n::t(app.lang, Key::RecentGuardLabel))
                        .size(13.0)
                        .color(palette.muted),
                ),
            );
            if ui
                .add(
                    egui::DragValue::new(&mut app.settings.watch.recent_guard_minutes)
                        .range(0..=10_080)
                        .speed(5.0),
                )
                .changed()
            {
                *dirty = true;
            }
        });
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ExclusionsTitle), |ui| {
        if widgets::text_input(
            ui,
            palette,
            &mut app.file_excludes,
            i18n::t(app.lang, Key::RootExcludeFiles),
            420.0,
        )
        .changed()
        {
            app.settings.watch.exclude_files = split_list(&app.file_excludes);
            *dirty = true;
        }

        ui.add_space(8.0);
        if widgets::text_input(
            ui,
            palette,
            &mut app.excludes,
            i18n::t(app.lang, Key::RootExcludeDirs),
            420.0,
        )
        .changed()
        {
            app.settings.watch.exclude_dirs = split_list(&app.excludes);
            *dirty = true;
        }
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::ExcludeHint));

        ui.add_space(8.0);
        if widgets::text_input(
            ui,
            palette,
            &mut app.temp_text,
            i18n::t(app.lang, Key::TempExtensionsLabel),
            420.0,
        )
        .changed()
        {
            app.settings.temp_extensions = split_list(&app.temp_text)
                .iter()
                .map(|ext| config::normalize_extension(ext))
                .filter(|ext| !ext.is_empty())
                .collect();
            *dirty = true;
        }

        ui.add_space(8.0);
        if widgets::text_input(
            ui,
            palette,
            &mut app.skip_text,
            i18n::t(app.lang, Key::SkipDirsLabel),
            420.0,
        )
        .changed()
        {
            app.settings.skip_dirs = split_list(&app.skip_text);
            *dirty = true;
        }
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::SystemFoldersNote));
    });
}

fn appearance(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(
        ui,
        palette,
        i18n::t(app.lang, Key::AppearanceTitle),
        i18n::t(app.lang, Key::AccentLabel),
    );

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ThemeLabel), |ui| {
        ui.horizontal(|ui| {
            for (value, key) in [
                (Theme::Dark, Key::ThemeDark),
                (Theme::Light, Key::ThemeLight),
                (Theme::System, Key::ThemeSystem),
            ] {
                if ui
                    .radio_value(&mut app.settings.theme, value, i18n::t(app.lang, key))
                    .changed()
                {
                    *dirty = true;
                }
            }
        });
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::AccentLabel), |ui| {
        ui.horizontal(|ui| {
            for choice in ACCENTS {
                let selected = app.settings.accent.eq_ignore_ascii_case(choice.value);
                if theme::accent_swatch(ui, palette, choice.value, selected).clicked() {
                    app.settings.accent = choice.value.to_string();
                    app.accent_draft = choice.value.to_string();
                    *dirty = true;
                }
                ui.label(
                    RichText::new(i18n::t(app.lang, choice.key))
                        .size(13.0)
                        .color(palette.muted),
                );
                ui.add_space(4.0);
            }
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            widgets::text_input(
                ui,
                palette,
                &mut app.accent_draft,
                i18n::t(app.lang, Key::AccentHexHint),
                110.0,
            );
            if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::ApplyColor)).clicked() {
                match config::normalize_color(&app.accent_draft) {
                    Some(value) => {
                        app.settings.accent = value.clone();
                        app.accent_draft = value;
                        *dirty = true;
                    }
                    None => {
                        app.notice = Some(super::Notice::error(
                            i18n::t(app.lang, Key::AccentInvalid).to_string(),
                        ));
                    }
                }
            }

            let mut rgb = theme::hex_rgb(&app.settings.accent);
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                app.settings.accent = theme::rgb_hex(rgb);
                app.accent_draft = app.settings.accent.clone();
                *dirty = true;
            }
        });

        let (fill, text) = theme::accent_pair(&app.settings.accent, ui.visuals().dark_mode);
        ui.horizontal(|ui| {
            egui::Frame::NONE
                .fill(palette.surface_alt)
                .corner_radius(egui::CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(RichText::new("Aa").size(13.0).color(text));
                });
            egui::Frame::NONE
                .fill(fill)
                .corner_radius(egui::CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(RichText::new("Aa").size(13.0).color(palette.on_accent));
                });
            ui.label(
                RichText::new(app.settings.accent.clone())
                    .size(13.0)
                    .color(palette.muted),
            );
            let ratio = theme::contrast(fill, egui::Color32::WHITE);
            ui.label(
                RichText::new(format!("{ratio:.1}:1"))
                    .size(13.0)
                    .color(if ratio >= 4.5 {
                        palette.success
                    } else {
                        palette.error
                    }),
            );
        });
        if theme::fill_contrast(&app.settings.accent) < 4.5 {
            ui.label(
                RichText::new(i18n::t(app.lang, Key::AccentFillWarn))
                    .size(13.0)
                    .color(palette.error),
            );
        }
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::TextSizeLabel), |ui| {
        ui.horizontal(|ui| {
            for (value, key) in [
                (TextSize::Small, Key::TextSmall),
                (TextSize::Normal, Key::TextNormal),
                (TextSize::Large, Key::TextLarge),
            ] {
                if ui
                    .radio_value(&mut app.settings.text_size, value, i18n::t(app.lang, key))
                    .changed()
                {
                    *dirty = true;
                }
            }
        });

        ui.add_space(8.0);
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::DensityLabel));

        ui.horizontal(|ui| {
            for (value, key) in [
                (Density::Compact, Key::DensityCompact),
                (Density::Comfortable, Key::DensityComfortable),
            ] {
                if ui
                    .radio_value(&mut app.settings.density, value, i18n::t(app.lang, key))
                    .changed()
                {
                    *dirty = true;
                }
            }
        });
    });
}

fn advanced(app: &mut App, ui: &mut egui::Ui, palette: &Palette, dirty: &mut bool) {
    header(
        ui,
        palette,
        i18n::t(app.lang, Key::AdvancedTitle),
        i18n::t(app.lang, Key::AdvancedHint),
    );

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::DataFolderTitle), |ui| {
        if let Ok(folder) = data_folder() {
            widgets::key_value(
                ui,
                palette,
                i18n::t(app.lang, Key::DataFolder),
                &folder.display().to_string(),
            );
            if widgets::link_button(ui, palette, i18n::t(app.lang, Key::OpenFolder)).clicked() {
                app.open_path(&folder);
            }
        }
        if let Ok(path) = config::config_path() {
            widgets::key_value(ui, palette, "config.toml", &path.display().to_string());
        }
        if let Ok(path) = config::journal_path() {
            widgets::key_value(ui, palette, "journal.jsonl", &path.display().to_string());
        }
        ui.add_space(6.0);
        widgets::key_value(
            ui,
            palette,
            i18n::t(app.lang, Key::VersionLabel),
            env!("CARGO_PKG_VERSION"),
        );
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ExportConfig), |ui| {
        ui.horizontal_wrapped(|ui| {
            if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::ExportConfig)).clicked() {
                app.export_config();
            }
            if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::ImportConfig)).clicked() {
                app.import_config();
            }
        });
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::ImportPreviewBody));
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::AdvancedMode), |ui| {
        if ui
            .checkbox(
                &mut app.settings.advanced,
                i18n::t(app.lang, Key::AdvancedMode),
            )
            .changed()
        {
            if !app.settings.advanced && app.tab == Tab::Organizar {
                app.tab = Tab::Ajustes;
            }
            *dirty = true;
        }
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::AdvancedHint));
        ui.add_space(6.0);
        if app.settings.advanced
            && widgets::link_button(ui, palette, i18n::t(app.lang, Key::TabOrganize)).clicked()
        {
            app.tab = Tab::Organizar;
        }
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::SubfoldersTitle), |ui| {
        ui.horizontal(|ui| {
            widgets::help_icon(ui, palette, i18n::t(app.lang, Key::TooltipSubfolders));
            widgets::small_muted(ui, palette, i18n::t(app.lang, Key::SubfoldersLabel));
        });
        let mut subfolders = app.settings.subfolders.clone();
        subfolder_controls(app, ui, &mut subfolders, 999, dirty);
        app.settings.subfolders = subfolders;
    });

    ui.add_space(12.0);

    widgets::card_titled(ui, palette, i18n::t(app.lang, Key::ResetTitle), |ui| {
        widgets::small_muted(ui, palette, i18n::t(app.lang, Key::ResetBody));
        ui.add_space(6.0);
        if widgets::danger_button(ui, palette, i18n::t(app.lang, Key::ResetSettings)).clicked() {
            app.confirm = Some(Confirm::Reset);
        }
    });
}

fn folder_edit_modal(app: &mut App, ctx: &egui::Context, palette: &Palette) {
    let Some(mut edit) = app.folder_edit.take() else {
        return;
    };

    let mut apply = false;
    let mut close = false;

    #[allow(deprecated)]
    let response = egui::Modal::new(egui::Id::new("folder-edit-modal")).show(ctx, |ui| {
        ui.set_width(400.0);
        ui.label(
            RichText::new(i18n::t(app.lang, Key::FolderRenameTitle))
                .size(17.0)
                .strong()
                .color(palette.text),
        );
        ui.add_space(8.0);
        widgets::text_input(
            ui,
            palette,
            &mut edit.value,
            i18n::t(app.lang, Key::CategoryFolderLabel),
            300.0,
        );
        if let Some(error) = &edit.error {
            ui.label(RichText::new(error).size(13.0).color(palette.error));
        }
        ui.add_space(14.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::accent_button(ui, palette, i18n::t(app.lang, Key::Apply)).clicked() {
                apply = true;
            }
            if widgets::ghost_button(ui, palette, i18n::t(app.lang, Key::Cancel)).clicked() {
                close = true;
            }
        });
    });

    if response.should_close() {
        close = true;
    }

    if apply {
        let new = edit.value.trim().to_string();
        let mut problem =
            config::folder_problem(&new).map(|problem| problem_text(app.lang, problem));

        if problem.is_none() {
            if let Some(categories) = app.scope_categories(edit.scope) {
                let duplicate = categories.iter().enumerate().any(|(index, category)| {
                    index != edit.index && category.folder.eq_ignore_ascii_case(&new)
                });
                if duplicate {
                    problem = Some(i18n::t(app.lang, Key::FolderProblemDuplicate).to_string());
                }
            }
        }

        if let Some(message) = problem {
            edit.error = Some(message);
        } else if let Some(mut categories) = app.scope_categories(edit.scope).cloned() {
            if let Some(category) = categories.get_mut(edit.index) {
                let old = category.folder.clone();
                category.folder = new.clone();
                let scope = edit.scope;
                app.set_scope_categories(scope, categories);
                app.save();

                let entries: usize = app
                    .scope_roots(scope)
                    .iter()
                    .map(|root| {
                        std::fs::read_dir(root.join(&old))
                            .map(|entries| entries.count())
                            .unwrap_or(0)
                    })
                    .sum();

                let only_case = cfg!(windows) && old.eq_ignore_ascii_case(&new);
                if only_case {
                    for root in app.scope_roots(scope) {
                        let _ = std::fs::rename(root.join(&old), root.join(&new));
                    }
                } else if old != new && entries > 0 {
                    app.confirm = Some(Confirm::FolderRename {
                        scope,
                        old,
                        new,
                        entries,
                    });
                }
                app.scan();
            }
            close = true;
        }
    }

    if close {
        app.folder_edit = None;
    } else {
        app.folder_edit = Some(edit);
    }
}

fn subfolder_controls(
    app: &App,
    ui: &mut egui::Ui,
    subfolders: &mut config::Subfolders,
    index: usize,
    dirty: &mut bool,
) {
    if ui
        .checkbox(
            &mut subfolders.by_year_month,
            i18n::t(app.lang, Key::SubByDate),
        )
        .changed()
    {
        *dirty = true;
    }
    if subfolders.by_year_month {
        egui::ComboBox::from_id_salt(("root-fecha", index))
            .selected_text(date_source_label(app.lang, subfolders.date_source))
            .width(180.0)
            .show_ui(ui, |ui| {
                for source in [DateSource::Modified, DateSource::Created, DateSource::Today] {
                    if ui
                        .selectable_value(
                            &mut subfolders.date_source,
                            source,
                            date_source_label(app.lang, source),
                        )
                        .changed()
                    {
                        *dirty = true;
                    }
                }
            });
    }
    if ui
        .checkbox(&mut subfolders.by_type, i18n::t(app.lang, Key::SubByType))
        .changed()
    {
        *dirty = true;
    }
    if ui
        .checkbox(
            &mut subfolders.by_capture,
            i18n::t(app.lang, Key::SubByCapture),
        )
        .changed()
    {
        *dirty = true;
    }
}

fn date_source_label(lang: Lang, source: DateSource) -> &'static str {
    match source {
        DateSource::Modified => i18n::t(lang, Key::DateModified),
        DateSource::Created => i18n::t(lang, Key::DateCreated),
        DateSource::Today => i18n::t(lang, Key::DateToday),
    }
}

fn problem_text(lang: Lang, problem: FolderProblem) -> String {
    let key = match problem {
        FolderProblem::Empty => Key::FolderProblemEmpty,
        FolderProblem::Invalid => Key::FolderProblemInvalid,
        FolderProblem::Reserved => Key::FolderProblemReserved,
        FolderProblem::Trailing => Key::FolderProblemTrailing,
        FolderProblem::TooLong => Key::FolderProblemTooLong,
    };
    i18n::t(lang, key).to_string()
}

fn split_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn profile_label(lang: Lang, profile: Profile) -> &'static str {
    let key = match profile {
        Profile::General => Key::ProfileGeneral,
        Profile::Student => Key::ProfileStudent,
        Profile::Developer => Key::ProfileDeveloper,
        Profile::Photographer => Key::ProfilePhotographer,
    };
    i18n::t(lang, key)
}
