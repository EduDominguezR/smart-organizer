//! Pruebas de la configuracion v2: migracion, condiciones, copias y raices.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use smart_organizer::config::{
    self, Action, Category, Condition, DateSource, Matcher, Root, Rule, Settings, Subfolders,
};
use smart_organizer::mover;
use smart_organizer::organize;
use smart_organizer::rules::{self, Engine, Target};
use smart_organizer::scanner;
use time::OffsetDateTime;

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

fn make_candidate(
    root: &Path,
    name: &str,
    modified: SystemTime,
    created: Option<SystemTime>,
) -> scanner::Candidate {
    scanner::Candidate {
        path: root.join(name),
        root: root.to_path_buf(),
        size: 128,
        modified,
        created,
        name: name.to_string(),
        ext: Path::new(name)
            .extension()
            .map(|ext| format!(".{}", ext.to_string_lossy().to_ascii_lowercase()))
            .unwrap_or_default(),
    }
}

#[test]
fn version_1_config_is_migrated_without_losing_rules() {
    let text = r##"
version = 1
theme = "dark"
accent = "#2563eb"
language = "es"
advanced = false
silent = false
background = false
all_in_one = false

[[roots]]
path = "C:\\Datos"
enabled = true

[[categories]]
name = "Otros"
folder = "Otros"
extensions = []
enabled = true

[[rules]]
id = 7
name = "Facturas"
enabled = true
matcher = { kind = "extension", value = ".pdf" }
action = { kind = "move_to", value = "Facturas" }

[subfolders]
by_year_month = false
by_type = false
by_capture = false

[watch]
debounce_ms = 1200
stability_ms = 500
recent_guard_minutes = 0
schedule_minutes = 0
"##;

    let settings = Settings::from_toml(text).expect("la version 1 debe cargar");
    assert_eq!(settings.version, config::CONFIG_VERSION);
    assert!(settings.categories[0].fallback, "Otros pasa a ser reserva");
    assert_eq!(settings.rules.len(), 1);
    assert_eq!(
        settings.rules[0].conditions,
        vec![Condition::Extension {
            value: ".pdf".to_string()
        }]
    );
    assert!(settings.rules[0].legacy_matcher.is_none());
    assert_eq!(settings.rules[0].name, "Facturas");
    assert_eq!(settings.rules[0].id, 7);
    assert!(settings.temp_extensions.contains(&".tmp".to_string()));
    assert!(settings.skip_dirs.iter().any(|name| name == "Windows"));
}

#[test]
fn invalid_config_is_rejected_without_panicking() {
    assert!(Settings::from_toml("version = ").is_err());
    assert!(Settings::from_toml("language = 42").is_err());
}

#[test]
fn glob_rules_ignore_case() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "a.pdf", b"pdf");

    let mut settings = settings_for(temp.path());
    settings.rules.push(Rule {
        conditions: vec![Condition::Glob {
            value: "*.PDF".to_string(),
        }],
        ..Rule::new(
            1,
            "Mayusculas",
            Action::MoveTo {
                value: "Globs".to_string(),
            },
        )
    });

    let plan = organize::build(&settings).expect("plan");
    let destination = plan
        .items
        .first()
        .and_then(|item| item.to.as_ref())
        .expect("destino")
        .display()
        .to_string();
    assert!(destination.contains("Globs"), "{destination}");
}

#[test]
fn date_source_uses_modified_created_or_today() {
    let temp = tempfile::tempdir().expect("temporal");
    let root = config::accept_root(temp.path()).expect("raiz");

    // 2021-03-05 12:00 UTC y 2019-07-10 12:00 UTC.
    let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_614_945_600);
    let created = SystemTime::UNIX_EPOCH + Duration::from_secs(1_562_764_800);
    let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("fecha");
    let sample = make_candidate(&root, "carta.pdf", modified, Some(created));

    let mut settings = Settings {
        roots: vec![Root::new(root.clone())],
        ..Settings::default()
    };
    settings.subfolders = Subfolders {
        by_year_month: true,
        date_source: DateSource::Modified,
        ..Subfolders::default()
    };

    let engine = Engine::new(&settings);
    let modified_target = engine.decide(&sample, now, 1);
    let Target::Move { dest, .. } = modified_target else {
        panic!("deberia moverse");
    };
    assert!(dest.ends_with(Path::new("Documentos/2021/03")));

    settings.subfolders.date_source = DateSource::Created;
    let engine = Engine::new(&settings);
    let Target::Move { dest, .. } = engine.decide(&sample, now, 1) else {
        panic!("deberia moverse");
    };
    assert!(dest.ends_with(Path::new("Documentos/2019/07")));

    settings.subfolders.date_source = DateSource::Today;
    let engine = Engine::new(&settings);
    let Target::Move { dest, .. } = engine.decide(&sample, now, 1) else {
        panic!("deberia moverse");
    };
    assert!(dest.ends_with(Path::new("Documentos/2023/11")));

    // Sin fecha de creacion se usa la de modificacion, nunca se descarta.
    let without_created = make_candidate(&root, "carta.pdf", modified, None);
    settings.subfolders.date_source = DateSource::Created;
    let engine = Engine::new(&settings);
    let Target::Move { dest, .. } = engine.decide(&without_created, now, 1) else {
        panic!("deberia moverse");
    };
    assert!(dest.ends_with(Path::new("Documentos/2021/03")));
}

#[test]
fn conditions_can_use_all_or_any() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "factura.pdf", b"pdf");

    let mut settings = settings_for(temp.path());
    let rule = |match_all: bool| Rule {
        conditions: vec![
            Condition::NameContains {
                value: "factura".to_string(),
            },
            Condition::Extension {
                value: ".png".to_string(),
            },
        ],
        match_all,
        ..Rule::new(
            1,
            "Combinada",
            Action::MoveTo {
                value: "Combinada".to_string(),
            },
        )
    };

    settings.rules.push(rule(false));
    let plan = organize::build(&settings).expect("plan con alguna");
    assert!(
        plan.items[0]
            .to
            .as_ref()
            .expect("destino")
            .display()
            .to_string()
            .contains("Combinada")
    );

    settings.rules[0] = rule(true);
    let plan = organize::build(&settings).expect("plan con todas");
    assert!(
        !plan.items[0]
            .to
            .as_ref()
            .expect("destino")
            .display()
            .to_string()
            .contains("Combinada")
    );
}

#[test]
fn copy_is_journaled_and_undo_removes_the_duplicate() {
    let temp = tempfile::tempdir().expect("temporal");
    let source = write(temp.path(), "nota.txt", b"contenido");
    let journal_dir = tempfile::tempdir().expect("diario");
    let journal_path = journal_dir.path().join("journal.jsonl");

    let mut settings = settings_for(temp.path());
    settings.rules.push(Rule {
        conditions: vec![Condition::Extension {
            value: ".txt".to_string(),
        }],
        ..Rule::new(
            1,
            "Copia",
            Action::CopyTo {
                value: "Copias".to_string(),
            },
        )
    });

    let plan = organize::build(&settings).expect("plan");
    assert_eq!(plan.items[0].op, organize::PlanOp::Copy);
    let summary = organize::execute_with(&plan, 0, &journal_path).expect("copiar");
    assert_eq!(summary.copied, 1);
    assert_eq!(summary.moved, 0);

    let copy = plan.items[0].to.as_ref().expect("destino").clone();
    assert!(source.exists(), "el original no se toca al copiar");
    assert!(copy.exists());

    let again = organize::build(&settings).expect("segundo plan");
    assert!(
        again.items.is_empty(),
        "una copia existente no se repite en cada pasada: {:?}",
        again
            .items
            .iter()
            .map(|item| (&item.from, &item.to, &item.category))
            .collect::<Vec<_>>()
    );

    let report = organize::undo_batch_with(&plan.batch, &journal_path).expect("deshacer copia");
    assert_eq!(report.restored, 1);
    assert!(source.exists());
    assert!(!copy.exists(), "la copia sale del lote al deshacer");
}

#[test]
fn name_templates_expand_and_sanitize() {
    let temp = tempfile::tempdir().expect("temporal");
    let root = config::accept_root(temp.path()).expect("raiz");
    let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_614_945_600);
    let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("fecha");
    let sample = make_candidate(&root, "informe final.pdf", modified, Some(modified));

    let name = rules::expand_template(
        "copia {original} {año}-{mes}-{dia} {contador} {categoria}",
        &sample,
        "Facturas",
        4,
        DateSource::Modified,
        now,
    );

    assert_eq!(name, "copia informe final 2021-03-05 4 Facturas.pdf");
}

#[test]
fn folder_names_are_validated_for_windows() {
    assert_eq!(
        config::folder_problem("CON"),
        Some(config::FolderProblem::Reserved)
    );
    assert_eq!(
        config::folder_problem("a:b"),
        Some(config::FolderProblem::Invalid)
    );
    assert_eq!(
        config::folder_problem("carpeta."),
        Some(config::FolderProblem::Trailing)
    );
    assert_eq!(
        config::folder_problem("   "),
        Some(config::FolderProblem::Empty)
    );
    assert!(config::folder_problem("Facturas 2026").is_none());
    assert_eq!(config::sanitize_folder("../Fuera"), "..-Fuera");
}

#[test]
fn rule_folders_can_never_escape_the_root() {
    let root = Path::new("C:\\Datos");
    for value in [
        "..\\..\\Windows",
        "C:\\Datos\\..\\Windows",
        "C:\\Windows",
        "..\\Datos\\..\\..\\Windows",
    ] {
        let resolved = config::resolve_within(root, value);
        assert!(
            resolved.starts_with(root),
            "{value} termino en {resolved:?}"
        );
        assert!(
            !resolved
                .components()
                .any(|component| { matches!(component, std::path::Component::ParentDir) })
        );
    }
    assert_eq!(
        config::resolve_within(root, "C:\\Datos\\Fotos"),
        root.join("Fotos")
    );
    assert_eq!(
        config::resolve_within(root, "..\\Windows"),
        root.join("Windows")
    );
}

#[test]
fn per_root_overrides_replace_categories_and_long_lived_lists() {
    let one = tempfile::tempdir().expect("temporal uno");
    let two = tempfile::tempdir().expect("temporal dos");
    let one = config::accept_root(one.path()).expect("raiz uno");
    let two = config::accept_root(two.path()).expect("raiz dos");

    let mut settings = Settings {
        roots: vec![Root::new(one.clone()), Root::new(two.clone())],
        ..Settings::default()
    };
    settings.roots[0].overrides.categories = Some(vec![
        Category {
            name: "Solo".to_string(),
            folder: "Solo".to_string(),
            extensions: vec![".abc".to_string()],
            enabled: true,
            fallback: false,
            color: None,
        },
        Category {
            name: "Otros".to_string(),
            folder: "Otros".to_string(),
            extensions: Vec::new(),
            enabled: true,
            fallback: true,
            color: None,
        },
    ]);
    settings.roots[0].overrides.exclude_files = vec!["*.bak".to_string()];

    let effective = settings.effective_for(&one);
    assert!(effective.categories.iter().any(|item| item.name == "Solo"));
    assert!(
        effective
            .watch
            .exclude_files
            .iter()
            .any(|glob| glob == "*.bak")
    );

    let global = settings.effective_for(&two);
    assert!(!global.categories.iter().any(|item| item.name == "Solo"));
}

#[test]
fn fallback_category_collects_unknown_extensions() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "raro.xyz", b"xyz");

    let settings = settings_for(temp.path());
    let plan = organize::build(&settings).expect("plan");
    let destination = plan.items[0]
        .to
        .as_ref()
        .expect("destino")
        .display()
        .to_string();
    assert!(destination.contains("Otros"));
}

#[test]
fn scanner_skips_only_the_destination_folder_at_the_root() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "Documentos/viejo.pdf", b"viejo");
    write(temp.path(), "sub/Documentos/propio.pdf", b"propio");
    write(temp.path(), "nuevo.pdf", b"nuevo");

    let settings = settings_for(temp.path());
    let candidates = scanner::scan_root(&settings, temp.path()).expect("escaneo");
    let names: Vec<String> = candidates
        .iter()
        .map(|candidate| candidate.path.display().to_string())
        .collect();

    assert_eq!(names.len(), 2, "{names:?}");
    assert!(
        names
            .iter()
            .any(|name| name.ends_with("sub\\Documentos\\propio.pdf")
                || name.ends_with("sub/Documentos/propio.pdf"))
    );
    assert!(names.iter().any(|name| name.ends_with("nuevo.pdf")));
}

#[test]
fn a_root_named_like_a_category_is_still_scanned() {
    let temp = tempfile::tempdir().expect("temporal");
    let root = temp.path().join("Documentos");
    fs::create_dir_all(&root).expect("carpeta");
    write(&root, "carta.pdf", b"pdf");

    let settings = settings_for(&root);
    let candidates = scanner::scan_root(&settings, &root).expect("escaneo");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].name, "carta.pdf");
}

#[test]
fn a_rule_destination_is_not_scanned_again() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "factura.pdf", b"pdf");

    let mut settings = settings_for(temp.path());
    settings.rules.push(Rule {
        conditions: vec![Condition::NameContains {
            value: "factura".to_string(),
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
    assert_eq!(plan.items.len(), 1);
    let journal_dir = tempfile::tempdir().expect("diario");
    organize::execute_with(&plan, 0, &journal_dir.path().join("journal.jsonl")).expect("mover");

    let again = organize::build(&settings).expect("segundo plan");
    assert!(
        again.items.is_empty(),
        "la carpeta destino de la regla no se recorre: {:?}",
        again
            .items
            .iter()
            .map(|item| (&item.from, &item.to, &item.category))
            .collect::<Vec<_>>()
    );
}

#[test]
fn name_conditions_ignore_case_and_accents() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "música.txt", b"texto");

    let mut settings = settings_for(temp.path());
    settings.rules.push(Rule {
        conditions: vec![Condition::NameContains {
            value: "MÚSICA".to_string(),
        }],
        ..Rule::new(
            1,
            "Musica",
            Action::MoveTo {
                value: "Sonidos".to_string(),
            },
        )
    });

    let plan = organize::build(&settings).expect("plan");
    assert_eq!(plan.items.len(), 1);
    assert!(
        plan.items[0]
            .to
            .as_ref()
            .expect("destino")
            .display()
            .to_string()
            .contains("Sonidos")
    );
}

#[test]
fn configurable_excludes_and_temp_extensions_are_applied() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "visible.txt", b"texto");
    write(temp.path(), "oculto.pdf", b"pdf");
    write(temp.path(), "descarga.custom", b"parcial");

    let mut settings = settings_for(temp.path());
    settings.watch.exclude_files = vec!["*.PDF".to_string()];
    settings.temp_extensions.push(".custom".to_string());

    let candidates = scanner::scan_root(&settings, temp.path()).expect("escaneo");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].name, "visible.txt");
}

#[test]
fn moving_a_folder_merges_without_overwriting() {
    let temp = tempfile::tempdir().expect("temporal");
    write(temp.path(), "vieja/a.txt", b"a");
    write(temp.path(), "nueva/a.txt", b"b");

    let moved = mover::move_folder(&temp.path().join("vieja"), &temp.path().join("nueva"))
        .expect("mover carpeta");
    assert_eq!(moved, 1);
    assert!(temp.path().join("nueva/a.txt").exists());
    assert!(temp.path().join("nueva/a (1).txt").exists());
    assert!(!temp.path().join("vieja").exists());
}

#[test]
fn legacy_matcher_condition_keeps_working_after_normalize() {
    let mut settings = Settings::default();
    settings.rules.push(Rule {
        legacy_matcher: Some(Matcher::Extension {
            value: "PDF".to_string(),
        }),
        ..Rule::new(1, "Vieja", Action::Trash)
    });
    settings.normalize();

    assert_eq!(
        settings.rules[0].conditions,
        vec![Condition::Extension {
            value: ".pdf".to_string()
        }]
    );
    assert!(settings.rules[0].legacy_matcher.is_none());
}

#[test]
fn reserved_folder_names_are_sanitized() {
    assert_eq!(config::sanitize_folder("CON"), "_CON");
    assert_eq!(config::sanitize_folder("nul.txt"), "_nul.txt");
    assert_eq!(config::sanitize_folder("Fotos"), "Fotos");
}

#[test]
fn a_rule_without_conditions_does_not_match() {
    let temp = tempfile::tempdir().expect("temporal");
    let candidate = make_candidate(temp.path(), "nota.txt", SystemTime::now(), None);
    let mut settings = settings_for(temp.path());
    settings.rules.push(Rule::new(
        1,
        "Sin condiciones",
        Action::MoveTo {
            value: "Destino".to_string(),
        },
    ));
    for category in &mut settings.categories {
        category.enabled = false;
    }

    let engine = Engine::new(&settings);
    let target = engine.decide(&candidate, OffsetDateTime::now_utc(), 1);
    assert!(matches!(target, Target::Leave(_)));
}

#[cfg(windows)]
#[test]
fn a_case_only_folder_rename_does_not_duplicate_files() {
    let temp = tempfile::tempdir().expect("temporal");
    let old = temp.path().join("Fotos");
    fs::create_dir_all(&old).expect("crear carpeta");
    write(&old, "foto.txt", b"contenido");
    let new = temp.path().join("fotos");

    mover::move_folder(&old, &new).expect("cambiar mayusculas");
    assert_eq!(
        fs::read(old.join("foto.txt")).expect("el archivo sigue"),
        b"contenido"
    );
    let names: Vec<String> = fs::read_dir(&old)
        .expect("leer carpeta")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(!names.iter().any(|name| name.contains("(1)")), "{names:?}");
}

#[cfg(windows)]
#[test]
fn category_folders_are_skipped_without_case_sensitivity() {
    let temp = tempfile::tempdir().expect("temporal");
    let root = temp.path();
    fs::create_dir_all(root.join("documentos")).expect("crear carpeta");
    write(&root.join("documentos"), "nota.txt", b"contenido");
    let settings = settings_for(root);

    let found =
        scanner::scan_root(&settings, &config::accept_root(root).expect("raiz")).expect("escanear");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_newer_config_version_is_rejected_instead_of_being_rewritten() {
    let text = "version = 99\n";
    let error = Settings::from_toml(text).expect_err("debe rechazar una version futura");
    assert!(error.contains("newer"), "{error}");
}

#[test]
fn defaults_use_normal_text_and_comfortable_density() {
    let settings = Settings::default();
    assert_eq!(settings.text_size, config::TextSize::Normal);
    assert_eq!(settings.density, config::Density::Comfortable);
}

#[test]
fn the_whole_home_profile_is_protected_but_its_subfolders_are_not() {
    let Some(home) = dirs::home_dir() else {
        return;
    };
    assert!(config::is_protected_root(&home), "{}", home.display());
    let inside = home.join("Documentos de prueba smart organizer");
    assert!(!config::is_protected_root(&inside), "{}", inside.display());
}

#[test]
fn new_window_settings_have_safe_defaults_and_limits() {
    let settings = Settings::default();
    assert!(!settings.onboarded);
    assert_eq!(settings.window_size, (1180.0, 720.0));
    assert_eq!(settings.window_pos, (-1.0, -1.0));

    let mut tiny = Settings {
        window_size: (100.0, 100.0),
        window_pos: (f32::NAN, 0.5),
        ..Settings::default()
    };
    tiny.normalize();
    assert_eq!(tiny.window_size, (900.0, 560.0));
    assert_eq!(tiny.window_pos, (-1.0, -1.0));
}

#[test]
fn preview_destination_uses_the_same_engine_as_the_scan() {
    let temp = tempfile::tempdir().expect("temporal");
    let root = config::accept_root(temp.path()).expect("carpeta valida");
    let mut settings = Settings {
        roots: vec![Root::new(root.clone())],
        categories: vec![Category::new("Facturas", "Facturas")],
        subfolders: Subfolders {
            by_year_month: false,
            by_type: false,
            by_capture: false,
            date_source: DateSource::Modified,
        },
        ..Settings::default()
    };
    let mut rule = Rule::new(
        1,
        "Facturas",
        Action::MoveTo {
            value: "Facturas".to_string(),
        },
    );
    rule.conditions = vec![Condition::Extension {
        value: ".pdf".to_string(),
    }];
    settings.rules = vec![rule];

    match rules::preview_destination(&settings, "factura.pdf") {
        Target::Move { dest, category, .. } => {
            assert_eq!(category, "Facturas");
            assert_eq!(dest, root.join("Facturas"));
        }
        _ => panic!("destino inesperado"),
    }

    let second = tempfile::tempdir().expect("temporal");
    let second_root = config::accept_root(second.path()).expect("segunda carpeta");
    settings.roots.push(Root::new(second_root.clone()));
    match rules::preview_destination_in(&settings, &second_root, "factura.pdf") {
        Target::Move { dest, .. } => {
            assert_eq!(dest, second_root.join("Facturas"));
        }
        _ => panic!("destino inesperado en la segunda carpeta"),
    }
}
