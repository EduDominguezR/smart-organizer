//! Un cierre forzado entre mover y anotar no debe dejar el lote sin deshacer.

use std::fs;

use serde_json::json;
use smart_organizer::journal::Journal;

fn pending_line(
    batch: &str,
    op: &str,
    from: &std::path::Path,
    to: &std::path::Path,
    size: u64,
) -> String {
    json!({
        "v": 1,
        "ts": "2026-01-01T00:00:00Z",
        "batch": batch,
        "state": "pending",
        "op": op,
        "from": from,
        "to": to,
        "size": size,
        "category": "Documentos"
    })
    .to_string()
}

#[test]
fn a_finished_move_is_recovered_as_undoable() {
    let temp = tempfile::tempdir().expect("temporal");
    let from = temp.path().join("origen.txt");
    let to = temp.path().join("Documentos").join("origen.txt");
    fs::create_dir_all(to.parent().expect("carpeta")).expect("crear carpeta");
    fs::write(&to, b"contenido").expect("escribir destino");
    let journal_path = temp.path().join("journal.jsonl");
    fs::write(
        &journal_path,
        pending_line("lote-1", "move", &from, &to, 9) + "\n",
    )
    .expect("escribir diario");

    let journal = Journal::open(journal_path).expect("abrir diario");
    let batches = journal.batch_summaries();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].files, 1);
    assert!(!batches[0].undone);
}

#[test]
fn a_trash_that_cannot_be_verified_stays_failed() {
    let temp = tempfile::tempdir().expect("temporal");
    let from = temp.path().join("borrado.txt");
    let journal_path = temp.path().join("journal.jsonl");
    fs::write(
        &journal_path,
        pending_line("lote-2", "trash", &from, &from, 0) + "\n",
    )
    .expect("escribir diario");

    let journal = Journal::open(journal_path).expect("abrir diario");
    assert!(journal.batch_summaries().is_empty());
}
