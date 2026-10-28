//! Los textos tienen que existir en los dos idiomas y no pueden llevar guion largo.

use smart_organizer::i18n::{self, ALL_KEYS, Lang};

#[test]
fn every_key_has_text_in_both_languages() {
    for &key in ALL_KEYS {
        for lang in [Lang::Es, Lang::En] {
            let text = i18n::t(lang, key);
            assert!(!text.trim().is_empty(), "{key:?} esta vacia en {lang:?}");
        }
    }
}

#[test]
fn no_text_uses_an_em_dash() {
    let mut texts: Vec<String> = ALL_KEYS
        .iter()
        .flat_map(|&key| [i18n::t(Lang::Es, key), i18n::t(Lang::En, key)])
        .map(str::to_string)
        .collect();

    texts.push(i18n::error_clock(Lang::Es).to_string());
    texts.push(i18n::error_clock(Lang::En).to_string());
    texts.push(i18n::cli_help(Lang::Es));
    texts.push(i18n::cli_help(Lang::En));

    for text in texts {
        assert!(!text.contains('\u{2014}'), "texto con guion largo: {text}");
    }
}

#[test]
fn english_text_uses_no_spanish_accents_except_template_placeholders() {
    for &key in ALL_KEYS {
        let text = i18n::t(Lang::En, key);
        let accented = text.chars().any(|ch| "áéíóúñÁÉÍÓÚÑ".contains(ch));
        if accented {
            assert!(
                text.contains("{año}"),
                "texto en ingles con acentos: {text}"
            );
        }
    }
}

#[test]
fn file_types_use_friendly_names() {
    assert_eq!(i18n::file_type_name(Lang::Es, ".pdf"), "PDF");
    assert_eq!(i18n::file_type_name(Lang::En, "docx"), "Word");
    assert_eq!(i18n::file_type_name(Lang::Es, ".jpeg"), "Imagen");
    assert_eq!(i18n::file_type_name(Lang::Es, ".psd"), "PSD");
    assert_eq!(
        i18n::file_types_summary(Lang::Es, &[".docx".to_string(), ".pdf".to_string()]),
        "PDF, Word"
    );
}
