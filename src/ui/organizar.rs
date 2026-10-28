//! Pantalla avanzada: el mismo editor de reglas de Ajustes, a pantalla completa.

use eframe::egui::{self, Align, Layout, RichText};

use super::reglas;
use super::theme::{self, Palette};
use super::widgets;
use super::{App, Scope, SettingsSection, Tab};
use crate::i18n::{self, Key};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::palette(ui, &app.settings);
    let mut dirty = false;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(i18n::t(app.lang, Key::AdvancedTitle))
                    .size(22.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(2.0);
            widgets::small_muted(ui, &palette, i18n::t(app.lang, Key::RuleOrderHint));
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::ghost_button(ui, &palette, i18n::t(app.lang, Key::TabSettings)).clicked() {
                app.tab = Tab::Ajustes;
                app.section = SettingsSection::Reglas;
            }
        });
    });

    ui.add_space(14.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            scope_rows(app, ui, &palette);
            ui.add_space(12.0);
            let scope = app.scope;
            reglas::editor(app, ui, &palette, scope, &mut dirty);
            ui.add_space(12.0);
        });

    if dirty {
        app.save();
    }
}

fn scope_rows(app: &mut App, ui: &mut egui::Ui, palette: &Palette) {
    let mut choices: Vec<(Scope, String)> = vec![(
        Scope::Global,
        i18n::t(app.lang, Key::ScopeGlobal).to_string(),
    )];

    for (index, root) in app.settings.roots.iter().enumerate() {
        if root.overrides.rules.is_some() {
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
            egui::vec2(90.0, 20.0),
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
        egui::ComboBox::from_id_salt("organizar-ambito")
            .selected_text(selected)
            .width(260.0)
            .show_ui(ui, |ui| {
                for (scope, name) in &choices {
                    ui.selectable_value(&mut app.scope, *scope, name.as_str());
                }
            });
        if app.scope != before {
            app.rule_draft = super::RuleDraft::default();
        }
    });
}
