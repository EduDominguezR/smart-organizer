//! El offset local se resuelve una vez y no cambia despues.

use smart_organizer::clock;

#[test]
fn init_is_idempotent_and_keeps_one_offset() {
    if let Err(err) = clock::init() {
        // En Linux, `time` no puede leer el offset si el proceso ya tiene hilos.
        // Ahi el test no aplica; en Windows siempre debe funcionar.
        #[cfg(not(windows))]
        {
            eprintln!("sin offset local en este entorno: {err}");
            return;
        }
        #[cfg(windows)]
        panic!("no se pudo resolver el offset local: {err}");
    }

    let first = clock::offset().expect("offset resuelto");
    assert!(clock::init().is_ok());
    assert_eq!(Some(first), clock::offset());

    let hours = i32::from(first.whole_hours());
    assert!(
        (-12..=14).contains(&hours),
        "offset fuera de rango: {hours}"
    );
}

#[test]
fn now_local_uses_the_resolved_offset() {
    if clock::init().is_err() {
        return;
    }

    let offset = clock::offset().expect("offset resuelto");
    let now = clock::now_local().expect("fecha local");

    assert_eq!(now.offset(), offset);
}
