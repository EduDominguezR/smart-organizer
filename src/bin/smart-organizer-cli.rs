//! Binario sin interfaz. Es el proceso que corre todo el dia como residente.

use std::process::ExitCode;

use smart_organizer::cli;
use smart_organizer::clock;
use smart_organizer::i18n::Lang;

fn main() -> ExitCode {
    if let Err(err) = clock::init() {
        eprintln!("{}", err.describe(Lang::default()));
        return ExitCode::FAILURE;
    }

    cli::run()
}
