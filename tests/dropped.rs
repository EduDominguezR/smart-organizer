use std::fs;

use smart_organizer::config;

#[test]
fn dropped_files_use_their_folder_and_reject_missing_paths() {
    let temp = tempfile::tempdir().expect("temp");
    let folder = temp.path().join("Descargas");
    fs::create_dir_all(&folder).expect("folder");
    let file = folder.join("factura.pdf");
    fs::write(&file, b"pdf").expect("file");
    let missing = temp.path().join("no-existe");

    let (accepted, rejected) = config::dropped_roots(&[file, folder.clone(), missing.clone()]);

    assert_eq!(accepted.len(), 1);
    assert_eq!(
        accepted[0],
        dunce::canonicalize(&folder).expect("canonical")
    );
    assert_eq!(rejected, vec![missing]);
}

#[test]
fn dropped_folder_roots_are_deduplicated() {
    let temp = tempfile::tempdir().expect("temp");
    let folder = temp.path().join("Escritorio");
    fs::create_dir_all(&folder).expect("folder");

    let (accepted, rejected) = config::dropped_roots(&[folder.clone(), folder.clone()]);

    assert_eq!(accepted.len(), 1);
    assert!(rejected.is_empty());
}
