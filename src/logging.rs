//! Registro de fallos y gancho de panico.
//!
//! Escribe en `error.log`, junto a los ajustes, y rota el archivo a 1 MB.
//! Si la carpeta de datos todavia no se puede resolver, usa la carpeta temporal
//! del sistema para no perder la traza.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;
use std::sync::Once;

use crate::config;
use crate::journal;

const MAX_LOG_BYTES: u64 = 1_048_576;
const LOG_NAME: &str = "error.log";

static HOOK: Once = Once::new();

/// Instala una sola vez el gancho que deja la traza en disco. No escribe en
/// consola: en release no hay consola y el archivo es la unica prueba.
pub fn install_panic_hook() {
    HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| {
            let payload = panic_payload(info);
            let location = info
                .location()
                .map(|location| format!("{}:{}", location.file(), location.line()))
                .unwrap_or_else(|| "desconocido".to_string());
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("sin nombre");
            let trace = std::backtrace::Backtrace::force_capture();
            append_block(&format!(
                "PANIC v{} hilo={name} lugar={location}\n{payload}\n{trace}",
                env!("CARGO_PKG_VERSION")
            ));
        }));
    });
}

/// Añade una linea al registro. Los fallos al escribir se ignoran: el llamador
/// nunca debe caer porque el registro no este disponible.
pub fn log_message(text: &str) {
    append_block(text);
}

/// Deja constancia de un panico capturado por `catch_unwind`.
pub fn log_panic(context: &str, payload: &(dyn std::any::Any + Send)) {
    let text = if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "panico sin mensaje".to_string()
    };
    append_block(&format!(
        "PANIC CAPTURADO v{} en {context}: {text}",
        env!("CARGO_PKG_VERSION")
    ));
}

fn panic_payload(info: &PanicHookInfo<'_>) -> String {
    if let Some(text) = info.payload().downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = info.payload().downcast_ref::<String>() {
        text.clone()
    } else {
        "panico sin mensaje".to_string()
    }
}

fn log_dir() -> PathBuf {
    config::config_dir().unwrap_or_else(|_| std::env::temp_dir().join("smart-organizer"))
}

fn append_block(text: &str) {
    let dir = log_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    rotate_if_needed(&dir);

    let path = dir.join(LOG_NAME);
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let _ = writeln!(
        file,
        "{} v{} {text}",
        journal::now_stamp(),
        env!("CARGO_PKG_VERSION")
    );
}

/// Conserva el archivo anterior como `error.log.1`. Si otro proceso lo esta
/// rotando a la vez, se sigue escribiendo en el archivo actual sin bloquear.
fn rotate_if_needed(dir: &std::path::Path) {
    let path = dir.join(LOG_NAME);
    let Ok(metadata) = fs::metadata(&path) else {
        return;
    };
    if metadata.len() < MAX_LOG_BYTES {
        return;
    }

    let previous = dir.join("error.log.1");
    let _ = fs::remove_file(&previous);
    let _ = fs::rename(&path, &previous);
}
