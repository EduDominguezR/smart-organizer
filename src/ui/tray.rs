//! Bandeja del sistema. Solo existe en Windows.
//!
//! Los eventos del menu llegan por un canal de `muda`. El manejador pide un
//! repintado y muestra la ventana antes de encolar la accion: asi la interfaz
//! despierta aunque se haya ocultado al cerrar.

use std::sync::{Mutex, OnceLock};

use eframe::egui;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::i18n::{self, Key, Lang};

use super::icon;

#[derive(Clone, Copy)]
pub enum TrayAction {
    Open,
    Toggle,
    Restart,
    Organize,
    Quit,
}

static PENDING: OnceLock<Mutex<Vec<TrayAction>>> = OnceLock::new();
static HANDLER: OnceLock<()> = OnceLock::new();

fn pending() -> &'static Mutex<Vec<TrayAction>> {
    PENDING.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn take_actions() -> Vec<TrayAction> {
    let mut actions = Vec::new();
    if let Ok(mut queue) = pending().lock() {
        actions.append(&mut queue);
    }
    actions
}

pub struct Tray {
    _icon: TrayIcon,
    lang: Lang,
}

impl Tray {
    pub fn new(ctx: &egui::Context, lang: Lang) -> Result<Self, String> {
        install_handler(ctx);
        Ok(Self {
            _icon: build(lang)?,
            lang,
        })
    }

    pub fn sync(&mut self, ctx: &egui::Context, lang: Lang) {
        if self.lang == lang {
            return;
        }
        if let Ok(new) = Self::new(ctx, lang) {
            *self = new;
        }
    }
}

fn build(lang: Lang) -> Result<TrayIcon, String> {
    let menu = Menu::new();
    for (id, key) in [
        ("open", Key::TrayOpen),
        ("toggle", Key::TrayToggle),
        ("restart", Key::RestartWatch),
        ("organize", Key::TrayOrganize),
        ("quit", Key::TrayQuit),
    ] {
        let item = MenuItem::with_id(id, i18n::t(lang, key), true, None);
        menu.append(&item).map_err(|err| err.to_string())?;
    }

    let icon = Icon::from_rgba(icon::rgba(), 32, 32).map_err(|err| err.to_string())?;

    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(i18n::t(lang, Key::AppName))
        .with_icon(icon)
        .build()
        .map_err(|err| err.to_string())
}

fn install_handler(ctx: &egui::Context) {
    if HANDLER.set(()).is_err() {
        return;
    }

    let ctx = ctx.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let action = match event.id.0.as_str() {
            "open" => Some(TrayAction::Open),
            "toggle" => Some(TrayAction::Toggle),
            "restart" => Some(TrayAction::Restart),
            "organize" => Some(TrayAction::Organize),
            "quit" => Some(TrayAction::Quit),
            _ => None,
        };
        let Some(action) = action else {
            return;
        };

        if let Ok(mut queue) = pending().lock() {
            queue.push(action);
        }

        if !matches!(action, TrayAction::Quit) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        ctx.request_repaint();
    }));
}
