//! Pruebas de punta a punta sobre carpetas temporales.
//!
//! Se usa `execute_with` para que el diario viva dentro del directorio temporal y
//! no se toque la configuracion real de quien corre los tests.

use std::fs;
use std::path::Path;

use smart_organizer::config::{self, Action, Condition, Root, Rule, Settings};
use smart_organizer::journal::{Journal, State};
use smart_organizer::organize;

fn settings_for(root: &Path) -> Settings {
    let canonical = config::accept_root(root).expect("carpeta temporal valida");
    Settings {
        roots: vec![Root::new(canonical)],
        ..Settings::default()
    }
}

fn write(root: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("crear carpeta padre");
    }
    fs::write(&path, bytes).expect("escribir archivo de prueba");
    path
}

#[test]
fn plan_routes_each_extension_to_its_category() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "informe.pdf", b"pdf");
    write(temp.path(), "foto.PNG", b"png");
    write(temp.path(), "cancion.mp3", b"mp3");
    write(temp.path(), "notas.txt", b"txt");

    let settings = settings_for(temp.path());
    let plan = organize::build(&settings).expect("plan");

    let destinations: Vec<String> = plan
        .items
        .iter()
        .filter_map(|item| item.to.as_ref())
        .map(|path| path.display().to_string())
        .collect();

    assert_eq!(plan.items.len(), 4);
    assert!(
        destinations
            .iter()
            .any(|path| Path::new(path).ends_with("Documentos/informe.pdf"))
    );
    assert!(
        destinations
            .iter()
            .any(|path| Path::new(path).ends_with("Imágenes/foto.PNG"))
    );
    assert!(
        destinations
            .iter()
            .any(|path| Path::new(path).ends_with("Música/cancion.mp3"))
    );
    assert!(
        destinations
            .iter()
            .any(|path| Path::new(path).ends_with("Documentos/notas.txt"))
    );
}

#[test]
fn execute_moves_files_and_never_overwrites() {
    let temp = tempfile::tempdir().expect("temporal");
    let source = write(temp.path(), "informe.pdf", b"nuevo");
    write(temp.path(), "Documentos/informe.pdf", b"viejo");

    let journal_path = temp.path().join("journal.jsonl");
    let settings = settings_for(temp.path());
    let plan = organize::build(&settings).expect("plan");

    let summary = organize::execute_with(&plan, 0, &journal_path).expect("ejecutar");
    assert_eq!(summary.moved, 1);
    assert!(!source.exists());

    let moved = temp.path().join("Documentos").join("informe (1).pdf");
    assert!(moved.exists(), "el archivo nuevo debe llevar sufijo");
    assert_eq!(
        fs::read(temp.path().join("Documentos").join("informe.pdf")).expect("original"),
        b"viejo"
    );
    assert_eq!(fs::read(moved).expect("movido"), b"nuevo");
}

#[test]
fn undo_brings_files_back_after_reopening_the_journal() {
    let temp = tempfile::tempdir().expect("temporal");
    let first = write(temp.path(), "a.txt", b"a");
    let second = write(temp.path(), "b.txt", b"b");
    let journal_path = temp.path().join("journal.jsonl");

    let settings = settings_for(temp.path());
    let plan = organize::build(&settings).expect("plan");
    let batch = plan.batch.clone();

    organize::execute_with(&plan, 0, &journal_path).expect("ejecutar");
    assert!(!first.exists());
    assert!(!second.exists());

    // Se abre de cero: como si la app se hubiera cerrado y vuelto a abrir.
    let journal = Journal::open(journal_path.clone()).expect("diario");
    assert!(
        journal
            .all()
            .iter()
            .any(|record| record.state == State::Done)
    );

    let report = organize::undo_batch_with(&batch, &journal_path).expect("deshacer");
    assert_eq!(report.restored, 2);
    assert_eq!(report.skipped, 0);

    assert!(first.exists());
    assert!(second.exists());

    let journal = Journal::open(journal_path).expect("diario reabierto");
    assert!(journal.batch_summaries().iter().all(|batch| batch.undone));
}

#[test]
fn rules_run_before_categories() {
    let temp = tempfile::tempdir().expect("temporal");
    let invoice = write(temp.path(), "factura.pdf", b"factura");

    let mut settings = settings_for(temp.path());
    settings.rules.push(Rule {
        conditions: vec![Condition::Extension {
            value: ".pdf".to_string(),
        }],
        ..Rule::new(
            1,
            "Facturas",
            Action::MoveTo {
                value: "Facturas".to_string(),
            },
        )
    });

    let plan = organize::build(&settings).expect("plan");
    let item = plan.items.first().expect("un archivo");

    assert_eq!(item.from, invoice);
    let destination = item.to.as_ref().expect("destino").display().to_string();
    assert!(destination.contains("Facturas"));
    assert!(!destination.contains("Documentos"));
}

#[test]
fn subfolders_by_year_month_and_type() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "foto.jpg", b"jpg");

    let mut settings = settings_for(temp.path());
    settings.subfolders.by_year_month = true;
    settings.subfolders.by_type = true;

    let plan = organize::build(&settings).expect("plan");
    let destination = plan
        .items
        .first()
        .and_then(|item| item.to.as_ref())
        .expect("destino")
        .display()
        .to_string();

    assert!(destination.contains("Imágenes"));
    assert!(destination.contains("JPG"));
}

#[test]
fn recent_files_are_left_alone() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "recien-descargado.zip", b"zip");

    let mut settings = settings_for(temp.path());
    settings.watch.recent_guard_minutes = 60;

    let plan = organize::build(&settings).expect("plan");
    assert!(plan.items.is_empty());
    assert_eq!(plan.left_alone, 1);
}
