//! Errores del nucleo.
//!
//! El enum no lleva texto: el mensaje se arma en `i18n` con el idioma activo, para
//! que la interfaz nunca muestre un `io::Error` crudo ni deje a la persona sin
//! saber que paso con sus archivos.

use std::fmt;
use std::path::PathBuf;

use crate::i18n::{self, Lang};

#[derive(Debug)]
pub enum Error {
    /// Fallo de disco mientras se trabajaba con esa ruta.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// El sistema no entrego el offset local al arrancar.
    ClockUnavailable,
    /// La ventana no pudo crearse o el backend grafico fallo.
    Gui { detail: String },
    /// La ventana no pudo crearse con OpenGL.
    GuiOpenGl { detail: String },
    /// Windows no acepto el registro de inicio automatico.
    Autostart { detail: String },
    /// El archivo de ajustes existe pero no se puede leer como tal.
    Config { path: PathBuf, detail: String },
    /// El diario no se pudo abrir o escribir.
    Journal { path: PathBuf, detail: String },
    /// Una ruta candidata cae fuera de las carpetas elegidas.
    OutsideRoots { path: PathBuf },
    /// La carpeta elegida es del sistema y no se toca.
    ProtectedFolder { path: PathBuf },
    /// No hay ninguna carpeta configurada.
    NoRoots,
    /// La ruta ya no existe cuando se fue a usar.
    NotFound { path: PathBuf },
    /// La copia entre discos no coincidio en tamano o hash.
    CopyVerify { path: PathBuf },
    /// La papelera del sistema rechazo el archivo.
    Trash { path: PathBuf, detail: String },
    /// El sistema no dijo donde guardar la configuracion.
    ConfigDirMissing,
    /// La vigilancia no pudo arrancar.
    Watch { detail: String },
}

impl Error {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn gui(detail: impl Into<String>) -> Self {
        Self::Gui {
            detail: detail.into(),
        }
    }

    pub fn config(path: impl Into<PathBuf>, detail: impl Into<String>) -> Self {
        Self::Config {
            path: path.into(),
            detail: detail.into(),
        }
    }

    pub fn journal(path: impl Into<PathBuf>, detail: impl Into<String>) -> Self {
        Self::Journal {
            path: path.into(),
            detail: detail.into(),
        }
    }

    pub fn trash(path: impl Into<PathBuf>, detail: impl Into<String>) -> Self {
        Self::Trash {
            path: path.into(),
            detail: detail.into(),
        }
    }

    /// Texto para la persona que usa la app, en su idioma.
    pub fn describe(&self, lang: Lang) -> String {
        match self {
            Self::Io { path, source } => i18n::error_io(lang, path, source),
            Self::ClockUnavailable => i18n::error_clock(lang).to_string(),
            Self::Gui { detail } => i18n::error_gui(lang, detail),
            Self::GuiOpenGl { detail } => i18n::error_gui_opengl(lang, detail),
            Self::Autostart { detail } => i18n::error_autostart(lang, detail),
            Self::Config { path, detail } => i18n::error_config(lang, path, detail),
            Self::Journal { path, detail } => i18n::error_journal(lang, path, detail),
            Self::OutsideRoots { path } => i18n::error_outside_roots(lang, path),
            Self::ProtectedFolder { path } => i18n::error_protected(lang, path),
            Self::NoRoots => i18n::error_no_roots(lang).to_string(),
            Self::NotFound { path } => i18n::error_not_found(lang, path),
            Self::CopyVerify { path } => i18n::error_copy_verify(lang, path),
            Self::Trash { path, detail } => i18n::error_trash(lang, path, detail),
            Self::ConfigDirMissing => i18n::error_config_dir(lang).to_string(),
            Self::Watch { detail } => i18n::error_watch(lang, detail),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Los logs van en espanol; la interfaz usa `describe` con su propio idioma.
        f.write_str(&self.describe(Lang::default()))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
