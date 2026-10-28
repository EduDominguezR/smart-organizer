//! Binario sin interfaz: organizacion puntual, simulacion, deshacer y vigilancia.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::config::{self, Root, Settings};
use crate::error::Error;
use crate::i18n::{self, Lang};
use crate::organize;
use crate::watcher;

pub fn run() -> ExitCode {
    let lang = Lang::default();

    match inner(lang) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.describe(lang));
            ExitCode::FAILURE
        }
    }
}

fn inner(lang: Lang) -> Result<ExitCode, Error> {
    let mut organize_path: Option<PathBuf> = None;
    let mut dry_run = false;
    let mut undo = false;
    let mut watch = false;

    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--organize" | "-o" => {
                let Some(value) = args.next() else {
                    eprintln!("{}", i18n::missing_value(lang, "--organize"));
                    return Ok(ExitCode::from(2));
                };
                organize_path = Some(PathBuf::from(value));
            }
            "--dry-run" => dry_run = true,
            "--undo-last" => undo = true,
            "--watch" => watch = true,
            "--help" | "-h" => {
                println!("{}", i18n::cli_help(lang));
                return Ok(ExitCode::SUCCESS);
            }
            "--version" | "-V" => {
                println!(
                    "{} {}",
                    i18n::t(lang, i18n::Key::AppName),
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(ExitCode::SUCCESS);
            }
            other => {
                eprintln!("{}", i18n::unknown_flag(lang, other));
                eprintln!();
                eprintln!("{}", i18n::cli_help(lang));
                return Ok(ExitCode::from(2));
            }
        }
    }

    let mut settings = match Settings::load() {
        Ok(settings) => settings,
        Err(err) => {
            // Un archivo de ajustes roto no impide organizar: se usan los valores seguros.
            eprintln!("{}", err.describe(lang));
            Settings::default()
        }
    };

    if let Some(path) = organize_path {
        let canonical = config::accept_root(&path)?;
        settings.roots = vec![Root::new(canonical)];
    }

    if undo {
        let report = organize::undo_last()?;
        print_undo(lang, &report);
        return Ok(ExitCode::SUCCESS);
    }

    if watch {
        // El CLI comparte el mutex con el vigilante oculto del ejecutable
        // grafico: nunca puede quedar mas de uno mirando la misma configuracion.
        #[cfg(windows)]
        let _watch_instance = {
            let name = config::instance_name("watch");
            let guard = single_instance::SingleInstance::new(&name).map_err(|err| {
                crate::error::Error::Watch {
                    detail: err.to_string(),
                }
            })?;
            if !guard.is_single() {
                return Ok(ExitCode::SUCCESS);
            }
            guard
        };
        watcher::run(settings)?;
        return Ok(ExitCode::SUCCESS);
    }

    if settings.active_roots().is_empty() {
        eprintln!("{}", i18n::error_no_roots(lang));
        return Ok(ExitCode::FAILURE);
    }

    let plan = organize::build(&settings)?;

    if plan.is_empty() {
        println!("{}", i18n::t(lang, i18n::Key::EmptyNoPlan));
        return Ok(ExitCode::SUCCESS);
    }

    for item in plan.selected() {
        println!("{}", organize::describe_item(item, lang));
    }
    println!();

    let destinations = plan.destinations();
    let destinations = if destinations.is_empty() {
        String::new()
    } else {
        destinations.join(", ")
    };
    println!(
        "{}",
        i18n::preview_summary(lang, plan.selected_count(), &destinations)
    );

    if dry_run {
        return Ok(ExitCode::SUCCESS);
    }

    let summary = organize::execute(&plan, settings.watch.stability_ms)?;
    println!(
        "{}",
        i18n::cli_moved(lang, summary.moved, summary.trashed, summary.bytes)
    );
    if summary.copied > 0 {
        println!("{}", i18n::cli_copied(lang, summary.copied));
    }

    for (path, reason) in &summary.failures {
        eprintln!("{}: {reason}", path.display());
    }

    Ok(ExitCode::SUCCESS)
}

fn print_undo(lang: Lang, report: &crate::journal::UndoReport) {
    if report.restored == 0 && report.skipped == 0 {
        println!("{}", i18n::t(lang, i18n::Key::NothingToUndo));
        return;
    }

    if report.skipped == 0 {
        println!("{}", i18n::undo_done(lang, report.restored));
        return;
    }

    println!(
        "{}",
        i18n::undo_partial(lang, report.restored, report.skipped)
    );
    for path in &report.skipped_paths {
        eprintln!("{}", path.display());
    }
}
