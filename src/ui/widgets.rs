//! Piezas visuales compartidas: tarjetas, botones, etiquetas, grafica y confirmacion.

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Rect, RichText, Sense,
    Stroke, Ui,
};

use super::theme::Palette;

pub fn card<R>(ui: &mut Ui, palette: &Palette, add: impl FnOnce(&mut Ui) -> R) -> R {
    palette
        .card()
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add(ui)
        })
        .inner
}

pub fn card_titled<R>(
    ui: &mut Ui,
    palette: &Palette,
    title: &str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    palette
        .card()
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, palette, title);
            ui.add_space(6.0);
            add(ui)
        })
        .inner
}

pub fn section_label(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(RichText::new(text).size(14.0).color(palette.text).strong());
}

/// Interruptor de dos estados. Sustituye a las casillas cuando la accion se
/// enciende o apaga; el estado no depende solo del color.
pub fn switch(
    ui: &mut Ui,
    palette: &Palette,
    value: &mut bool,
    label: &str,
    lang: crate::i18n::Lang,
) -> egui::Response {
    let desired = egui::vec2(40.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());
    let activated = response.has_focus()
        && ui.input(|input| {
            input.key_pressed(egui::Key::Space) || input.key_pressed(egui::Key::Enter)
        });
    if response.clicked() || activated {
        *value = !*value;
        response.mark_changed();
    }

    let track = if *value {
        palette.accent
    } else {
        palette.surface_alt
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(11), track);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(11),
        Stroke::new(
            1.0,
            if response.has_focus() {
                palette.focus
            } else {
                palette.border
            },
        ),
        egui::StrokeKind::Inside,
    );
    let radius = 8.0;
    let center = if *value {
        rect.right_center() - egui::vec2(11.0, 0.0)
    } else {
        rect.left_center() + egui::vec2(11.0, 0.0)
    };
    ui.painter()
        .circle_filled(center, radius, palette.on_accent);

    if !label.is_empty() {
        ui.add_space(8.0);
        let text = ui.label(RichText::new(label).size(13.0).color(palette.text));
        response = response.union(text);
    }
    let state = if *value {
        crate::i18n::t(lang, crate::i18n::Key::SwitchOn)
    } else {
        crate::i18n::t(lang, crate::i18n::Key::SwitchOff)
    };
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::Switch,
            true,
            *value,
            format!("{label}, {state}"),
        )
    });
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Control segmentado para dos o tres opciones excluyentes.
pub fn segmented(
    ui: &mut Ui,
    palette: &Palette,
    options: &[&str],
    selected: usize,
) -> Option<usize> {
    let mut picked = None;
    egui::Frame::NONE
        .fill(palette.surface_alt)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (index, text) in options.iter().enumerate() {
                    let active = index == selected;
                    let response = ui.add(
                        egui::Button::new(RichText::new(*text).size(13.0).color(if active {
                            palette.on_accent
                        } else {
                            palette.text
                        }))
                        .fill(if active {
                            palette.accent
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(4))
                        .min_size(egui::vec2(92.0, 30.0)),
                    );
                    if response.clicked() {
                        picked = Some(index);
                    }
                }
            });
        });
    picked
}

#[derive(Clone, Copy)]
pub enum ActionIcon {
    Undo,
    Organize,
}

/// Boton compacto con icono dibujado, para ventanas estrechas. El tooltip
/// conserva el nombre de la accion.
pub fn icon_action(
    ui: &mut Ui,
    palette: &Palette,
    icon: ActionIcon,
    enabled: bool,
    tooltip: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::click());
    let color = if enabled { palette.text } else { palette.muted };
    if response.hovered() && enabled {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), palette.surface_alt);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, palette.focus),
            egui::StrokeKind::Inside,
        );
    }
    let painter = ui.painter();
    match icon {
        ActionIcon::Undo => {
            let center = rect.center();
            let radius = 8.0;
            let start = std::f32::consts::PI * 1.15;
            let points: Vec<egui::Pos2> = (0..20)
                .map(|step| {
                    let angle = start + step as f32 / 19.0 * std::f32::consts::PI * 1.55;
                    center + egui::vec2(angle.cos(), angle.sin()) * radius
                })
                .collect();
            painter.add(egui::Shape::line(points, Stroke::new(1.7, color)));
            let tip = center + egui::vec2(-radius * 0.94, -radius * 0.15);
            painter.add(egui::Shape::convex_polygon(
                vec![tip, tip + egui::vec2(5.0, 1.5), tip + egui::vec2(1.0, 6.0)],
                color,
                Stroke::NONE,
            ));
        }
        ActionIcon::Organize => {
            let center = rect.center() + egui::vec2(1.0, 0.0);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    center + egui::vec2(-5.0, -8.0),
                    center + egui::vec2(8.0, 0.0),
                    center + egui::vec2(-5.0, 8.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
    }
    response.on_hover_text(tooltip)
}

pub fn small_muted(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(RichText::new(text).size(13.0).color(palette.muted));
}

pub fn accent_button(ui: &mut Ui, palette: &Palette, text: &str) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).color(palette.on_accent).strong())
        .fill(palette.accent)
        .corner_radius(CornerRadius::same(6))
        .min_size(egui::vec2(0.0, 36.0));
    ui.add(button)
}

pub fn accent_button_enabled(
    ui: &mut Ui,
    palette: &Palette,
    text: &str,
    enabled: bool,
) -> egui::Response {
    if enabled {
        return accent_button(ui, palette, text);
    }

    let button = egui::Button::new(RichText::new(text).color(palette.muted))
        .fill(palette.surface_alt)
        .corner_radius(CornerRadius::same(6))
        .min_size(egui::vec2(0.0, 36.0));
    ui.add_enabled(false, button)
}

pub fn ghost_button(ui: &mut Ui, palette: &Palette, text: &str) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).color(palette.text))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(4))
        .min_size(egui::vec2(0.0, 36.0));
    ui.add(button)
}

pub fn ghost_button_enabled(
    ui: &mut Ui,
    palette: &Palette,
    text: &str,
    enabled: bool,
) -> egui::Response {
    if enabled {
        return ghost_button(ui, palette, text);
    }
    let button = egui::Button::new(RichText::new(text).color(palette.muted))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(4))
        .min_size(egui::vec2(0.0, 36.0));
    ui.add_enabled(false, button)
}

pub fn danger_button(ui: &mut Ui, palette: &Palette, text: &str) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).color(palette.error))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, palette.error))
        .corner_radius(CornerRadius::same(4))
        .min_size(egui::vec2(0.0, 36.0));
    ui.add(button)
}

pub fn link_button(ui: &mut Ui, palette: &Palette, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(13.0).color(palette.accent_text))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 28.0)),
    )
}

pub fn pill(ui: &mut Ui, palette: &Palette, text: &str, color: Color32) {
    egui::Frame::NONE
        .fill(palette.tint(color))
        .stroke(Stroke::new(1.0, palette.tint(color)))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(9, 6))
        .show(ui, |ui| {
            ui.set_min_height(16.0);
            ui.label(RichText::new(text).size(13.0).color(color));
        });
}

pub fn stat(ui: &mut Ui, palette: &Palette, label: &str, value: &str, hint: &str) {
    ui.vertical(|ui| {
        section_label(ui, palette, label);
        ui.add_space(2.0);
        ui.label(RichText::new(value).size(24.0).strong().color(palette.text));
        if !hint.is_empty() {
            ui.add_space(2.0);
            small_muted(ui, palette, hint);
        }
    });
}

pub fn divider(ui: &mut Ui, palette: &Palette) {
    ui.add_space(4.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, palette.hairline());
    ui.add_space(4.0);
}

pub fn bar_chart(
    ui: &mut Ui,
    palette: &Palette,
    data: &[(String, usize)],
    rows: usize,
    empty: &str,
    color_of: impl Fn(&str) -> Color32,
) {
    if data.is_empty() {
        small_muted(ui, palette, empty);
        return;
    }

    let max = data
        .iter()
        .map(|(_, count)| *count)
        .max()
        .unwrap_or(1)
        .max(1);

    for (name, count) in data.iter().take(rows) {
        ui.horizontal(|ui| {
            let (name_rect, _) =
                ui.allocate_exact_size(egui::vec2(96.0, 20.0), egui::Sense::hover());
            ui.painter().text(
                name_rect.left_center(),
                Align2::LEFT_CENTER,
                name,
                FontId::proportional(13.0),
                palette.muted,
            );

            let width = (ui.available_width() - 44.0).max(40.0);
            let (bar_rect, _) =
                ui.allocate_exact_size(egui::vec2(width, 20.0), egui::Sense::hover());
            let track = Rect::from_min_size(
                bar_rect.left_center() - egui::vec2(0.0, 4.0),
                egui::vec2(width, 8.0),
            );
            ui.painter()
                .rect_filled(track, CornerRadius::same(4), palette.surface_alt);
            let filled = Rect::from_min_size(
                track.min,
                egui::vec2(track.width() * (*count as f32 / max as f32), track.height()),
            );
            ui.painter()
                .rect_filled(filled, CornerRadius::same(4), color_of(name));

            let (count_rect, _) =
                ui.allocate_exact_size(egui::vec2(36.0, 20.0), egui::Sense::hover());
            ui.painter().text(
                count_rect.right_center(),
                Align2::RIGHT_CENTER,
                count.to_string(),
                FontId::proportional(13.0),
                palette.text,
            );
        });
    }
}

pub fn empty_state(
    ui: &mut Ui,
    palette: &Palette,
    title: &str,
    body: &str,
    add: impl FnOnce(&mut Ui),
) {
    ui.add_space(36.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(18.0).strong().color(palette.text));
        ui.add_space(6.0);
        ui.label(RichText::new(body).size(13.0).color(palette.muted));
        ui.add_space(16.0);
        add(ui);
    });
}

pub fn folder_chip(ui: &mut Ui, palette: &Palette, text: &str) {
    egui::Frame::NONE
        .fill(palette.surface_alt)
        .stroke(Stroke::new(1.0, palette.hairline()))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.set_min_height(18.0);
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot.center(), 3.0, palette.success);
                ui.label(RichText::new(text).size(13.0).color(palette.text));
            });
        });
}

pub fn key_value(ui: &mut Ui, palette: &Palette, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::vec2(170.0, 28.0),
            egui::Label::new(RichText::new(key).size(13.0).color(palette.muted)),
        );
        ui.label(RichText::new(value).size(13.0).color(palette.text));
    });
}

pub fn folder_icon(ui: &mut Ui, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::hover());
    let painter = ui.painter();
    let body = Rect::from_min_max(rect.min + egui::vec2(0.0, 3.0), rect.max);
    let tab = Rect::from_min_size(rect.min + egui::vec2(2.0, 0.0), egui::vec2(7.0, 5.0));
    painter.rect_filled(tab, CornerRadius::same(2), palette.accent_text);
    painter.rect_filled(body, CornerRadius::same(3), palette.accent_text);
}

pub fn help_icon(ui: &mut Ui, palette: &Palette, tooltip: &str) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::hover());
    let center = rect.center();
    ui.painter()
        .circle_stroke(center, 10.5, Stroke::new(1.0, palette.muted));
    ui.painter().text(
        center,
        Align2::CENTER_CENTER,
        "?",
        FontId::proportional(15.0),
        palette.muted,
    );
    response.on_hover_text(tooltip);
}

pub fn menu_button(ui: &mut Ui, palette: &Palette) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), palette.surface_alt);
    }
    let center = rect.center();
    for offset in [-6.0, 0.0, 6.0] {
        ui.painter()
            .circle_filled(center + egui::vec2(0.0, offset), 1.7, palette.muted);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Entrada de navegacion lateral: sin borde cuando esta inactiva, relleno y
/// barra de acento cuando esta activa. El foco de teclado tambien se ve.
pub fn nav_entry(ui: &mut Ui, palette: &Palette, text: &str, selected: bool) -> egui::Response {
    let width = ui.available_width();
    let response = ui.add_sized(
        egui::vec2(width, 36.0),
        egui::Button::new(RichText::new(text).size(13.0).color(if selected {
            palette.text
        } else {
            palette.muted
        }))
        .fill(if selected {
            palette.tint(palette.accent)
        } else {
            Color32::TRANSPARENT
        })
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(4)),
    );

    if selected {
        let bar = Rect::from_min_size(
            response.rect.min + egui::vec2(0.0, 7.0),
            egui::vec2(3.0, response.rect.height() - 14.0),
        );
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), palette.accent);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.shrink(1.0),
            CornerRadius::same(4),
            Stroke::new(1.0, palette.accent),
            egui::StrokeKind::Inside,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Confirmacion modal. Devuelve Some(true) si se confirma, Some(false) si se
/// cierra o se cancela, None mientras siga abierta.
pub fn modal_confirm(
    ctx: &egui::Context,
    palette: &Palette,
    title: &str,
    body: &str,
    confirm: &str,
    cancel: &str,
) -> Option<bool> {
    let mut answer = None;

    // `Id::new` sigue siendo la via para un id estable de ventana; la alternativa
    // nueva (IdSalt) no la acepta Modal todavia.
    #[allow(deprecated)]
    let modal_id = egui::Id::new("confirm-modal");

    let response = egui::Modal::new(modal_id).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.label(RichText::new(title).size(17.0).strong().color(palette.text));
        ui.add_space(8.0);
        ui.label(RichText::new(body).size(13.0).color(palette.muted));
        ui.add_space(18.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if accent_button(ui, palette, confirm).clicked() {
                answer = Some(true);
            }
            if ghost_button(ui, palette, cancel).clicked() {
                answer = Some(false);
            }
        });
    });

    if response.should_close() {
        answer = Some(false);
    }

    answer
}

pub fn text_input(
    ui: &mut Ui,
    palette: &Palette,
    value: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(palette.muted))
            .desired_width(width)
            .margin(egui::Margin::symmetric(8, 7)),
    )
}
