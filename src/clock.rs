//! Zona horaria local.
//!
//! `time` solo puede leer el offset en el hilo principal y sin otros hilos activos.
//! Por eso se resuelve una vez en `main`, antes de lanzar cualquier worker, y queda
//! en una variable global de solo lectura.

use std::sync::OnceLock;

use time::{OffsetDateTime, UtcOffset};

use crate::error::Error;

static OFFSET: OnceLock<UtcOffset> = OnceLock::new();

/// Resuelve el offset local. Se puede llamar varias veces; solo la primera hace trabajo.
pub fn init() -> Result<(), Error> {
    if OFFSET.get().is_some() {
        return Ok(());
    }

    let offset = UtcOffset::current_local_offset().map_err(|_| Error::ClockUnavailable)?;
    let _ = OFFSET.set(offset);

    Ok(())
}

pub fn offset() -> Option<UtcOffset> {
    OFFSET.get().copied()
}

/// Fecha local actual. Devuelve `None` si nadie llamo a `init` antes.
pub fn now_local() -> Option<OffsetDateTime> {
    OFFSET
        .get()
        .map(|offset| OffsetDateTime::now_utc().to_offset(*offset))
}
