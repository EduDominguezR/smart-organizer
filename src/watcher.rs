//! Proceso residente: vigila las carpetas y organiza cuando algo llega.
//!
//! notify avisa de los cambios; aqui se agrupan para no organizar veinte veces
//! mientras un archivo se sigue copiando. La estabilidad de tamano la comprueba
//! `organize::execute` antes de mover nada.
//!
//! El bucle tiene tres defensas: un tope de espera para no quedarse agrupando
//! para siempre, un filtro para los eventos de las carpetas destino y de los
//! movimientos propios, y una recarga de `config.toml` antes de cada pasada.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime};

use notify::{Event, RecursiveMode, Watcher};

use crate::config::{self, Action, Settings};
use crate::error::Error;
use crate::i18n::{self, Lang};
use crate::journal;
use crate::organize;

type Paths = Vec<PathBuf>;

enum Wake {
    Changed(Paths),
    Config,
}

pub fn run(mut settings: Settings) -> Result<(), Error> {
    crate::priority::lower_process();
    crate::priority::lower_thread();
    // Solo el vigilante acepta EcoQoS. La ventana mantiene su velocidad normal.
    crate::priority::eco_qos();
    let _ = config::clear_watch_stop();
    let mut roots = settings.active_roots();
    if roots.is_empty() {
        return Err(Error::NoRoots);
    }

    let (sender, receiver) = mpsc::channel::<Wake>();
    let (mut watcher, mut destinations, mut protected) =
        build_watcher(&settings, &roots, sender.clone())?;
    let _config_watcher = build_config_watcher(sender.clone());

    let mut lang = language(&settings);
    let mut debounce = Duration::from_millis(settings.watch.debounce_ms);
    let mut schedule = schedule_of(&settings);
    let mut last_run = Instant::now();
    let mut config_stamp = config_stamp();

    println!("{}", i18n::cli_watching_start(lang, roots.len()));

    loop {
        if config::watch_stop_requested() {
            let _ = config::clear_watch_stop();
            return Ok(());
        }

        // Sin nada que hacer el proceso duerme en el canal. El temporizador
        // solo existe si hay una organizacion periodica configurada.
        let next =
            schedule.map(|period| (last_run + period).saturating_duration_since(Instant::now()));
        let received = match next {
            Some(wait) => match receiver.recv_timeout(wait) {
                Ok(wake) => Some(wake),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            },
            None => match receiver.recv() {
                Ok(wake) => Some(wake),
                Err(_) => return Ok(()),
            },
        };

        let Some(first) = received else {
            reload(
                &mut settings,
                &mut roots,
                &mut watcher,
                &mut destinations,
                &mut protected,
                &mut lang,
                &mut debounce,
                &mut schedule,
                &mut config_stamp,
                &sender,
            );
            organize_once(&settings, lang, &[]);
            last_run = Instant::now();
            continue;
        };

        match first {
            Wake::Config => {
                reload(
                    &mut settings,
                    &mut roots,
                    &mut watcher,
                    &mut destinations,
                    &mut protected,
                    &mut lang,
                    &mut debounce,
                    &mut schedule,
                    &mut config_stamp,
                    &sender,
                );
            }
            Wake::Changed(first) => {
                let mut changed = first;
                // Un archivo grande genera muchos eventos. Se espera a que
                // callen, pero nunca mas de diez debounces ni treinta segundos.
                let cap = debounce
                    .saturating_mul(10)
                    .min(Duration::from_secs(30))
                    .max(debounce);
                let deadline = Instant::now() + cap;
                loop {
                    let now = Instant::now();
                    if now >= deadline {
                        break;
                    }
                    // Se organiza tras un periodo de silencio; el tope evita
                    // que un flujo continuo de eventos lo posponga sin fin.
                    let wait = debounce.min(deadline - now);
                    match receiver.recv_timeout(wait) {
                        Ok(Wake::Changed(more)) => changed.extend(more),
                        Ok(Wake::Config) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
                    }
                }

                reload(
                    &mut settings,
                    &mut roots,
                    &mut watcher,
                    &mut destinations,
                    &mut protected,
                    &mut lang,
                    &mut debounce,
                    &mut schedule,
                    &mut config_stamp,
                    &sender,
                );

                let own = organize_once(&settings, lang, &changed);
                // Los eventos del propio lote ya no interesan. Si mientras
                // tanto llego algo distinto, se vuelve a disparar una pasada.
                if drain_after(&receiver, &own, &destinations, &protected) {
                    let _ = sender.send(Wake::Changed(Vec::new()));
                }
                last_run = Instant::now();
            }
        }
    }
}

/// Crea el watcher y registra todas las raices. Devuelve tambien los filtros
/// que usa el callback para no despertar por sus propios destinos.
fn build_watcher(
    settings: &Settings,
    roots: &[PathBuf],
    sender: Sender<Wake>,
) -> Result<(notify::RecommendedWatcher, Vec<String>, HashSet<String>), Error> {
    let destinations = destination_dirs(settings);
    let filtered_destinations = destinations.clone();
    let protected = protected_files();
    let filtered_protected = protected.clone();

    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        let event = match event {
            Ok(event) => event,
            Err(err) => {
                log_error(Lang::default(), &format!("notify: {err}"));
                return;
            }
        };
        if event
            .paths
            .iter()
            .any(|path| !ignored(path, &filtered_destinations, &filtered_protected))
        {
            let _ = sender.send(Wake::Changed(event.paths));
        }
    })
    .map_err(|err| Error::Watch {
        detail: err.to_string(),
    })?;

    for root in roots {
        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|err| Error::Watch {
                detail: err.to_string(),
            })?;
    }

    Ok((watcher, destinations, protected))
}

/// Avisa cuando se guarda `config.toml` para no despertar cada medio segundo
/// solo para preguntar si cambio. Vigila la carpeta, pero filtra el nombre.
fn build_config_watcher(sender: Sender<Wake>) -> Option<notify::RecommendedWatcher> {
    let path = config::config_path().ok()?;
    let parent = path.parent()?.to_path_buf();
    let wanted = key(&path);
    let stop = key(&parent.join("watch.stop"));
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        if let Ok(event) = event
            && event.paths.iter().any(|path| {
                let path = key(path);
                path == wanted || path == stop
            })
        {
            let _ = sender.send(Wake::Config);
        }
    })
    .ok()?;
    watcher.watch(&parent, RecursiveMode::NonRecursive).ok()?;
    Some(watcher)
}

#[allow(clippy::too_many_arguments)]
fn reload(
    settings: &mut Settings,
    roots: &mut Vec<PathBuf>,
    watcher: &mut notify::RecommendedWatcher,
    destinations: &mut Vec<String>,
    protected: &mut HashSet<String>,
    lang: &mut Lang,
    debounce: &mut Duration,
    schedule: &mut Option<Duration>,
    known_stamp: &mut Option<SystemTime>,
    sender: &Sender<Wake>,
) {
    let Some(stamp) = config_stamp() else {
        return;
    };
    if *known_stamp == Some(stamp) {
        return;
    }
    *known_stamp = Some(stamp);

    let new_settings = match Settings::load() {
        Ok(settings) => settings,
        Err(err) => {
            log_error(*lang, &err.describe(*lang));
            return;
        }
    };

    let new_roots = new_settings.active_roots();
    if new_roots.is_empty() {
        log_error(*lang, i18n::error_no_roots(*lang));
        return;
    }

    match build_watcher(&new_settings, &new_roots, sender.clone()) {
        Ok((new_watcher, new_destinations, new_protected)) => {
            *watcher = new_watcher;
            *roots = new_roots;
            *destinations = new_destinations;
            *protected = new_protected;
            *lang = language(&new_settings);
            *debounce = Duration::from_millis(new_settings.watch.debounce_ms);
            *schedule = schedule_of(&new_settings);
            *settings = new_settings;
        }
        Err(err) => log_error(*lang, &err.describe(*lang)),
    }
}

fn schedule_of(settings: &Settings) -> Option<Duration> {
    match settings.watch.schedule_minutes {
        0 => None,
        minutes => Some(Duration::from_secs(u64::from(minutes) * 60)),
    }
}

fn language(settings: &Settings) -> Lang {
    if settings.language == "en" {
        Lang::En
    } else {
        Lang::default()
    }
}

fn config_stamp() -> Option<SystemTime> {
    let path = config::config_path().ok()?;
    fs::metadata(path).ok()?.modified().ok()
}

/// Carpetas que reciben archivos. Cualquier evento por debajo de una de ellas
/// es consecuencia de una pasada anterior, no una descarga nueva.
fn destination_dirs(settings: &Settings) -> Vec<String> {
    let mut dirs = Vec::new();
    for root in settings.active_roots() {
        let effective = settings.effective_for(&root);
        for category in &effective.categories {
            if !category.enabled {
                continue;
            }
            dirs.push(key(&effective.category_folder(&root, category)));
        }
        for rule in effective.rules.iter().filter(|rule| rule.enabled) {
            let base = match &rule.action {
                Action::MoveTo { value } | Action::CopyTo { value } => Some(value.as_str()),
                Action::MoveRename { folder, .. } | Action::CopyRename { folder, .. } => {
                    Some(folder.as_str())
                }
                _ => None,
            };
            if let Some(value) = base {
                dirs.push(key(&config::resolve_within(&root, value)));
            }
        }
    }
    dirs.sort();
    dirs.dedup();
    dirs
}

/// Archivos de la propia app que no deben disparar una pasada, sobre todo en
/// modo portable, cuando viven dentro de una carpeta vigilada.
fn protected_files() -> HashSet<String> {
    let mut files = HashSet::new();
    if let Ok(dir) = config::config_dir() {
        for name in [
            "config.toml",
            "config.bak",
            "journal.jsonl",
            "error.log",
            "portable",
        ] {
            files.insert(key(&dir.join(name)));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        files.insert(key(&exe));
    }
    files
}

fn ignored(path: &Path, destinations: &[String], protected: &HashSet<String>) -> bool {
    let path = key(path);
    if protected.contains(&path) {
        return true;
    }
    destinations
        .iter()
        .any(|dir| path == *dir || path.starts_with(&format!("{dir}\\")))
}

/// Consume lo que quedo en la cola despues de organizar. Devuelve true si
/// habia un evento ajeno al lote; en ese caso hay que volver a mirar.
fn drain_after(
    receiver: &Receiver<Wake>,
    own: &[PathBuf],
    destinations: &[String],
    protected: &HashSet<String>,
) -> bool {
    let own: HashSet<String> = own.iter().map(|path| key(path)).collect();
    let mut residual = false;
    while let Ok(wake) = receiver.try_recv() {
        if let Wake::Changed(paths) = wake
            && paths
                .iter()
                .any(|path| !own.contains(&key(path)) && !ignored(path, destinations, protected))
        {
            residual = true;
        }
    }
    residual
}

/// Una pasada nunca debe tumbar el proceso residente. Si el plan o un
/// movimiento entran en panico, se anota, se repara el diario y se conserva el
/// proceso para que la proxima pasada pueda seguir.
fn organize_once(settings: &Settings, lang: Lang, changed: &[PathBuf]) -> Vec<PathBuf> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        organize_pass(settings, lang, changed)
    })) {
        Ok(own) => own,
        Err(payload) => {
            crate::logging::log_panic("vigilante durante una pasada", payload.as_ref());
            recover_after_panic(lang);
            // Los eventos de esta pasada ya se consumieron: no se reintentan en
            // bucle. Los archivos siguen en disco y la proxima pasada los vera.
            changed.to_vec()
        }
    }
}

fn recover_after_panic(lang: Lang) {
    let recovered = std::panic::catch_unwind(journal::recover_pending);
    match recovered {
        Ok(Ok(())) => {}
        Ok(Err(err)) => crate::logging::log_message(&format!(
            "No se pudo reparar el diario tras un panico: {}",
            err.describe(lang)
        )),
        Err(payload) => crate::logging::log_panic("al reparar el diario", payload.as_ref()),
    }
}

fn organize_pass(settings: &Settings, lang: Lang, changed: &[PathBuf]) -> Vec<PathBuf> {
    let plan = match if changed.is_empty() {
        organize::build(settings)
    } else {
        organize::build_paths(settings, changed)
    } {
        Ok(plan) => plan,
        Err(err) => {
            log_error(lang, &err.describe(lang));
            return Vec::new();
        }
    };

    if plan.is_empty() {
        return Vec::new();
    }

    let own: Vec<PathBuf> = plan.selected().map(|item| item.from.clone()).collect();

    match organize::execute(&plan, settings.watch.stability_ms) {
        Ok(summary) if summary.moved + summary.copied + summary.trashed > 0 => {
            println!(
                "{}",
                i18n::cli_moved(lang, summary.moved, summary.trashed, summary.bytes)
            );
            if summary.copied > 0 {
                println!("{}", i18n::cli_copied(lang, summary.copied));
            }
            for (path, reason) in summary.failures {
                eprintln!("{}: {reason}", path.display());
                log_error(lang, &format!("{}: {reason}", path.display()));
            }
        }
        Ok(_) => {}
        Err(err) => log_error(lang, &err.describe(lang)),
    }

    own
}

fn log_error(_lang: Lang, text: &str) {
    crate::logging::log_message(text);
}

/// Clave de comparacion de rutas. Windows no distingue mayusculas y acepta
/// ambas barras; el resto de sistemas conserva la comparacion exacta.
fn key(path: &Path) -> String {
    let text = path.to_string_lossy().replace('/', "\\");
    #[cfg(windows)]
    {
        text.trim_end_matches('\\').to_lowercase()
    }
    #[cfg(not(windows))]
    {
        text.trim_end_matches('\\').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_events_are_ignored_but_new_files_are_not() {
        let root = PathBuf::from("C:\\Users\\demo\\Downloads");
        let destinations = vec![key(&root.join("Imagenes"))];
        let protected = HashSet::new();
        assert!(ignored(
            &root.join("Imagenes").join("foto.png"),
            &destinations,
            &protected
        ));
        assert!(!ignored(&root.join("nueva.txt"), &destinations, &protected));
    }

    #[test]
    fn the_debounce_cap_is_bounded() {
        let debounce = Duration::from_millis(1200);
        let cap = debounce
            .saturating_mul(10)
            .min(Duration::from_secs(30))
            .max(debounce);
        assert_eq!(cap, Duration::from_secs(12));
    }
}
