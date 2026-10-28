//! Arranque de la interfaz y del vigilante oculto.
//!
//! En release no hay consola: si algo falla antes de abrir la ventana, el error
//! se escribe en `error.log` junto a los ajustes y se muestra en un cuadro del
//! sistema. El modo `--watch` no abre ventana ni cuadros: solo escribe el fallo
//! en el mismo registro.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;

use smart_organizer::clock;
use smart_organizer::config::{self, Settings};
use smart_organizer::error::Error;
use smart_organizer::i18n::{self, Lang};
use smart_organizer::journal;
use smart_organizer::logging;
use smart_organizer::ui;
use smart_organizer::watcher;

fn main() -> ExitCode {
    logging::install_panic_hook();

    // La consciencia de DPI viene del manifiesto de Windows. Llamarla aqui
    // antes de winit puede dejar el primer tamano de ventana en pixeles
    // fisicos y recortar la interfaz en pantallas con escala.
    let watch = std::env::args().skip(1).any(|arg| arg == "--watch");
    let name = config::instance_name(if watch { "watch" } else { "gui" });

    // La instancia unica se toma antes de leer nada: un segundo doble clic sale
    // en silencio aunque la configuracion este a medio escribir.
    let _instance = match acquire(&name) {
        Ok(Some(guard)) => guard,
        Ok(None) => {
            if !watch {
                focus_first_window();
            }
            return ExitCode::SUCCESS;
        }
        Err(detail) => {
            logging::log_message(&detail);
            return ExitCode::FAILURE;
        }
    };

    let fallback = detected_language();
    let settings = match Settings::load() {
        Ok(settings) => settings,
        Err(err) => {
            if watch {
                return fail_watch(fallback, &err);
            }
            return fail(fallback, &err);
        }
    };
    let lang = language(&settings);

    if let Err(err) = clock::init() {
        return if watch {
            fail_watch(lang, &err)
        } else {
            fail(lang, &err)
        };
    }

    if let Err(err) = journal::recover_pending() {
        logging::log_message(&format!(
            "No se pudo revisar el diario al arrancar: {}",
            err.describe(lang)
        ));
    }

    if watch {
        return match watcher::run(settings) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => fail_watch(lang, &err),
        };
    }

    match ui::run(lang) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail_gui(lang, &err),
    }
}

/// Un fallo al crear la ventana no es lo mismo que un fallo de configuracion.
/// Se muestra una explicacion util y se registra con el detalle tecnico.
fn fail_gui(lang: Lang, err: &Error) -> ExitCode {
    let text = i18n::t(lang, i18n::Key::GuiStartFailed);
    eprintln!("{text}");
    logging::log_message(&format!("{text} Detalle: {}", err.describe(lang)));
    message_box(text);
    ExitCode::FAILURE
}

fn language(settings: &Settings) -> Lang {
    if settings.language == "en" {
        Lang::En
    } else {
        Lang::default()
    }
}

/// Idioma del sistema cuando `config.toml` no se pudo leer. En Windows se usa
/// el idioma de la interfaz del usuario; fuera de Windows, `LANG`.
#[cfg(windows)]
fn detected_language() -> Lang {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }

    let id = unsafe { GetUserDefaultUILanguage() };
    if id & 0x03ff == 0x0a {
        Lang::Es
    } else {
        Lang::En
    }
}

#[cfg(not(windows))]
fn detected_language() -> Lang {
    match std::env::var("LANG") {
        Ok(value) if value.to_lowercase().starts_with("es") => Lang::Es,
        Ok(_) => Lang::En,
        Err(_) => Lang::default(),
    }
}

#[cfg(windows)]
fn acquire(name: &str) -> Result<Option<single_instance::SingleInstance>, String> {
    let guard = single_instance::SingleInstance::new(name)
        .map_err(|err| format!("No se pudo crear la instancia unica: {err}"))?;
    if guard.is_single() {
        Ok(Some(guard))
    } else {
        Ok(None)
    }
}

#[cfg(not(windows))]
fn acquire(_name: &str) -> Result<Option<()>, String> {
    Ok(Some(()))
}

/// Un segundo doble clic no abre otra ventana: recupera la que ya existe,
/// tambien si estaba oculta en la bandeja.
#[cfg(windows)]
fn focus_first_window() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn FindWindowW(class: *const u16, title: *const u16) -> isize;
        fn ShowWindow(window: isize, command: i32) -> i32;
        fn SetForegroundWindow(window: isize) -> i32;
    }

    const SW_RESTORE: i32 = 9;

    let title: Vec<u16> = OsStr::new("Smart Organizer")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    for _ in 0..20 {
        let window = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if window != 0 {
            unsafe {
                ShowWindow(window, SW_RESTORE);
                SetForegroundWindow(window);
            }
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[cfg(not(windows))]
fn focus_first_window() {}

/// Windows arranca el proceso como DPI-unaware si el binario no lleva manifiesto.
/// Sin esto, la ventana se dibuja a 1.25x sobre un lienzo que no lo es y el
/// contenido se sale por la derecha. En otros sistemas no hace falta.
fn fail(lang: Lang, err: &Error) -> ExitCode {
    let text = err.describe(lang);
    eprintln!("{text}");
    logging::log_message(&text);
    message_box(&text);
    ExitCode::FAILURE
}

/// El modo oculto no puede abrir un cuadro que nadie vera. Registra el fallo y
/// sale; los archivos no se tocan.
fn fail_watch(lang: Lang, err: &Error) -> ExitCode {
    let text = err.describe(lang);
    eprintln!("{text}");
    logging::log_message(&text);
    ExitCode::FAILURE
}

#[cfg(windows)]
fn message_box(text: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            window: *mut core::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            kind: u32,
        ) -> i32;
    }

    const MB_OK: u32 = 0x0000_0000;
    const MB_ICONERROR: u32 = 0x0000_0010;

    let text: Vec<u16> = OsStr::new(text)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let caption: Vec<u16> = OsStr::new("Smart Organizer")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // El puntero nulo es una ventana sin padre: el cuadro sale centrado en pantalla.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
fn message_box(_text: &str) {}
