//! Paleta y estilo de la interfaz.
//!
//! Los colores salen de DESIGN.md y estan medidos con contraste AA sobre el fondo
//! que les toca. `#2563eb` solo se usa como relleno con texto blanco; como texto
//! sobre fondo oscuro se usa su version aclarada.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

use crate::config::{Density, Settings, Theme};
use crate::i18n::Key;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    /// Texto legible encima del relleno de acento (blanco o casi negro).
    pub on_accent: Color32,
    pub error: Color32,
    pub success: Color32,
    pub border: Color32,
    pub focus: Color32,
}

impl Palette {
    pub fn for_mode(dark: bool, accent: &str) -> Self {
        let (accent, accent_text) = accent_pair(accent, dark);
        let mut palette = if dark { Self::dark() } else { Self::light() };
        palette.accent = accent;
        palette.accent_text = accent_text;
        palette.on_accent = better_text(accent);
        // El foco de teclado sigue al acento elegido, no a un azul fijo.
        palette.focus = accent;
        palette
    }

    fn dark() -> Self {
        Self {
            bg: hex("#171717", Color32::BLACK),
            surface: hex("#1f1f1f", Color32::BLACK),
            surface_alt: hex("#262626", Color32::BLACK),
            text: hex("#e8e8e8", Color32::WHITE),
            muted: hex("#a3a3a3", Color32::GRAY),
            accent: hex("#2563eb", Color32::BLUE),
            accent_text: hex("#60a5fa", Color32::LIGHT_BLUE),
            on_accent: Color32::WHITE,
            error: hex("#f87171", Color32::RED),
            success: hex("#22c55e", Color32::GREEN),
            border: hex("#8a8a8a", Color32::GRAY),
            focus: hex("#93c5fd", Color32::LIGHT_BLUE),
        }
    }

    fn light() -> Self {
        Self {
            bg: hex("#f7f7f8", Color32::WHITE),
            surface: hex("#ffffff", Color32::WHITE),
            surface_alt: hex("#efeff1", Color32::WHITE),
            text: hex("#171717", Color32::BLACK),
            muted: hex("#525252", Color32::DARK_GRAY),
            accent: hex("#2563eb", Color32::BLUE),
            accent_text: hex("#1d4ed8", Color32::BLUE),
            on_accent: Color32::WHITE,
            error: hex("#b91c1c", Color32::RED),
            success: hex("#15803d", Color32::GREEN),
            border: hex("#737373", Color32::GRAY),
            focus: hex("#2563eb", Color32::BLUE),
        }
    }

    /// Linea de separacion: el borde con muy poca opacidad.
    pub fn hairline(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.border.r(), self.border.g(), self.border.b(), 48)
    }

    /// Relleno tenue del mismo tono, para etiquetas y avisos.
    pub fn tint(&self, color: Color32) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 36)
    }

    pub fn card(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.surface)
            .stroke(Stroke::new(1.0, self.hairline()))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(egui::Margin::same(16))
    }
}

pub fn palette(ui: &egui::Ui, settings: &Settings) -> Palette {
    Palette::for_mode(ui.visuals().dark_mode, &settings.accent)
}

pub fn hex(value: &str, fallback: Color32) -> Color32 {
    let trimmed = value.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return fallback;
    }

    let parsed = u32::from_str_radix(trimmed, 16);
    let Ok(number) = parsed else {
        return fallback;
    };

    Color32::from_rgb(
        ((number >> 16) & 0xff) as u8,
        ((number >> 8) & 0xff) as u8,
        (number & 0xff) as u8,
    )
}

fn mix(from: Color32, to: Color32, amount: f32) -> Color32 {
    let blend =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * amount).round() as u8;
    Color32::from_rgb(
        blend(from.r(), to.r()),
        blend(from.g(), to.g()),
        blend(from.b(), to.b()),
    )
}

/// Devuelve el color de relleno y un tono legible para texto sobre el fondo.
///
/// El tono se acerca al blanco (tema oscuro) o al negro (tema claro) hasta que
/// alcanza 4.5:1 contra el fondo real. No se toca el relleno elegido.
pub fn accent_pair(value: &str, dark: bool) -> (Color32, Color32) {
    let fill = hex(value, hex("#2563eb", Color32::BLUE));
    let background = if dark {
        hex("#262626", Color32::BLACK)
    } else {
        hex("#efeff1", Color32::WHITE)
    };
    let toward = if dark { Color32::WHITE } else { Color32::BLACK };

    let mut text = fill;
    for step in 0..=20 {
        let candidate = mix(fill, toward, step as f32 / 20.0);
        text = candidate;
        if contrast(candidate, background) >= 4.5 {
            break;
        }
    }

    (fill, text)
}

/// Texto blanco o casi negro, el que mejor contraste tenga sobre el relleno.
pub fn better_text(fill: Color32) -> Color32 {
    let white = contrast(fill, Color32::WHITE);
    let black = contrast(fill, Color32::BLACK);
    if white >= black {
        Color32::WHITE
    } else {
        Color32::from_rgb(17, 17, 17)
    }
}

/// Contraste de un relleno con texto blanco. Si no llega a 4.5:1, la interfaz
/// avisa y usa texto oscuro en los botones.
pub fn fill_contrast(value: &str) -> f32 {
    contrast(hex(value, Color32::BLUE), Color32::WHITE)
}

pub fn contrast(a: Color32, b: Color32) -> f32 {
    let first = luminance(a);
    let second = luminance(b);
    let (light, dark) = if first > second {
        (first, second)
    } else {
        (second, first)
    };
    (light + 0.05) / (dark + 0.05)
}

fn luminance(color: Color32) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

pub fn hex_rgb(value: &str) -> [u8; 3] {
    let color = hex(value, hex("#2563eb", Color32::BLUE));
    [color.r(), color.g(), color.b()]
}

pub fn rgb_hex(rgb: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

pub struct AccentChoice {
    pub key: Key,
    pub value: &'static str,
}

pub const ACCENTS: &[AccentChoice] = &[
    AccentChoice {
        key: Key::AccentBlue,
        value: "#2563eb",
    },
    AccentChoice {
        key: Key::AccentGreen,
        value: "#15803d",
    },
    AccentChoice {
        key: Key::AccentPurple,
        value: "#6d28d9",
    },
    AccentChoice {
        key: Key::AccentAmber,
        value: "#b45309",
    },
];

/// Instala la tipografia del sistema en Windows. Si no esta, egui usa la suya.
pub fn install_system_fonts(ctx: &egui::Context) {
    #[cfg(windows)]
    {
        let mut fonts = egui::FontDefinitions::default();
        let mut changed = false;

        if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
            fonts.font_data.insert(
                "segoe-ui".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "segoe-ui".to_owned());
            changed = true;
        }

        if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\consola.ttf") {
            fonts.font_data.insert(
                "consolas".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "consolas".to_owned());
            changed = true;
        }

        if changed {
            ctx.set_fonts(fonts);
        }
    }

    #[cfg(not(windows))]
    {
        let _ = ctx;
    }
}

pub fn apply(ctx: &egui::Context, settings: &Settings) {
    let preference = match settings.theme {
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
        Theme::System => egui::ThemePreference::System,
    };
    ctx.set_theme(preference);
    // El zoom escala tambien los tamaños explicitos de RichText, asi que el
    // tamaño de texto elegido afecta a toda la interfaz desde un solo sitio.
    ctx.set_zoom_factor(settings.text_size.scale());

    let theme = ctx.theme();
    let mut style = (*ctx.style_of(theme)).clone();
    let dark = style.visuals.dark_mode;
    let palette = Palette::for_mode(dark, &settings.accent);

    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.override_text_color = Some(palette.text);
    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.surface_alt;
    visuals.faint_bg_color = palette.surface_alt;
    visuals.code_bg_color = palette.surface_alt;
    visuals.hyperlink_color = palette.accent_text;
    visuals.error_fg_color = palette.error;
    visuals.warn_fg_color = palette.accent_text;
    visuals.window_stroke = Stroke::new(1.0, palette.hairline());
    visuals.window_corner_radius = CornerRadius::same(4);
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;

    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.hairline());
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    visuals.widgets.inactive.bg_fill = palette.surface_alt;
    visuals.widgets.inactive.weak_bg_fill = palette.surface_alt;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    visuals.widgets.hovered.bg_fill = palette.surface_alt;
    visuals.widgets.hovered.weak_bg_fill = palette.surface_alt;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.muted);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    visuals.widgets.active.bg_fill = palette.tint(palette.accent);
    visuals.widgets.active.weak_bg_fill = palette.tint(palette.accent);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    visuals.selection.bg_fill = palette.accent;
    visuals.selection.stroke = Stroke::new(1.0, palette.on_accent);
    visuals.striped = true;

    style.visuals = visuals;
    let comfortable = settings.density == Density::Comfortable;
    style.spacing.item_spacing = egui::vec2(10.0, if comfortable { 8.0 } else { 4.0 });
    style.spacing.button_padding = egui::vec2(14.0, if comfortable { 8.0 } else { 5.0 });
    style.spacing.interact_size = egui::vec2(32.0, if comfortable { 32.0 } else { 28.0 });
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.menu_margin = egui::Margin::same(8);

    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(18.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(13.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();

    ctx.set_style_of(theme, style);
}

/// Alto de una fila de tabla segun la densidad elegida.
pub fn row_height(settings: &Settings) -> f32 {
    if settings.density == Density::Compact {
        32.0
    } else {
        42.0
    }
}

/// Un cuadrado de color con el borde de foco, para elegir acento.
pub fn accent_swatch(
    ui: &mut egui::Ui,
    palette: &Palette,
    value: &str,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let color = hex(value, palette.accent);
    ui.painter().rect_filled(rect, CornerRadius::same(4), color);

    if selected {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            CornerRadius::same(4),
            Stroke::new(2.0, palette.focus),
            egui::StrokeKind::Outside,
        );
    }

    response
}
