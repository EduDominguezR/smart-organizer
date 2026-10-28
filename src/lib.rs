//! Nucleo compartido por la interfaz y por el binario de consola.
//!
//! La interfaz nunca toca el disco directamente: pide trabajo al hilo de fondo y
//! recibe resultados por el canal de comandos. Este crate no sabe nada de egui.

pub mod cli;
pub mod clock;
pub mod config;
pub mod duplicates;
pub mod error;
pub mod i18n;
pub mod journal;
pub mod known_folders;
pub mod logging;
pub mod mover;
pub mod organize;
pub mod priority;
pub mod rules;
pub mod scanner;
pub mod watcher;

#[cfg(feature = "gui")]
pub mod ui;
