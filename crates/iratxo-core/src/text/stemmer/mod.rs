//! Language-aware stemmer dispatch.
//!
//! - English / Spanish / French / Italian / German / Dutch: delegated to
//!   `rust-stemmers`, which implements the Snowball algorithms (the same
//!   family marrow uses).
//! - Catalan: hand-rolled light stemmer (rust-stemmers does not ship Catalan).
//! - Basque: hand-port of marrow's stemmer (no upstream Rust crate exists).

mod basque;
mod catalan;
mod snowball_word;

use crate::text::lang::Language;
use rust_stemmers::{Algorithm, Stemmer};
use std::borrow::Cow;
use std::sync::OnceLock;

static EN_STEMMER: OnceLock<Stemmer> = OnceLock::new();
static ES_STEMMER: OnceLock<Stemmer> = OnceLock::new();
static FR_STEMMER: OnceLock<Stemmer> = OnceLock::new();
static IT_STEMMER: OnceLock<Stemmer> = OnceLock::new();
static DE_STEMMER: OnceLock<Stemmer> = OnceLock::new();
static NL_STEMMER: OnceLock<Stemmer> = OnceLock::new();

fn snowball(lang: Language) -> Option<&'static Stemmer> {
    Some(match lang {
        Language::English => EN_STEMMER.get_or_init(|| Stemmer::create(Algorithm::English)),
        Language::Spanish => ES_STEMMER.get_or_init(|| Stemmer::create(Algorithm::Spanish)),
        Language::French  => FR_STEMMER.get_or_init(|| Stemmer::create(Algorithm::French)),
        Language::Italian => IT_STEMMER.get_or_init(|| Stemmer::create(Algorithm::Italian)),
        Language::German  => DE_STEMMER.get_or_init(|| Stemmer::create(Algorithm::German)),
        Language::Dutch   => NL_STEMMER.get_or_init(|| Stemmer::create(Algorithm::Dutch)),
        Language::Catalan | Language::Basque => return None,
    })
}

pub fn stem(word: &str, lang: Language) -> String {
    if word.is_empty() { return String::new(); }
    match lang {
        Language::Catalan => catalan::stem(word),
        Language::Basque  => basque::stem(word),
        _ => snowball(lang).expect("snowball language").stem(word).into_owned(),
    }
}

/// Same as [`stem`] but returns `Cow<str>` to avoid allocation when the
/// stemmer returns the input unchanged (common for short/irregular words).
pub fn stem_cow(word: &str, lang: Language) -> Cow<'_, str> {
    if word.is_empty() { return Cow::Borrowed(""); }
    match lang {
        Language::Catalan => Cow::Owned(catalan::stem(word)),
        Language::Basque  => Cow::Owned(basque::stem(word)),
        _ => snowball(lang).expect("snowball language").stem(word),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_stems_match_porter2() {
        assert_eq!(stem("running",   Language::English), "run");
        assert_eq!(stem("cancelling", Language::English), "cancel");
        assert_eq!(stem("agreements", Language::English), "agreement");
        assert_eq!(stem("policies",   Language::English), "polici"); // Snowball English drops -es to -i
    }

    #[test]
    fn spanish_stems_match_snowball() {
        assert_eq!(stem("corriendo", Language::Spanish), "corr");
        assert_eq!(stem("hablando",  Language::Spanish), "habl");
        assert_eq!(stem("políticas", Language::Spanish), "polit");
    }

    #[test]
    fn catalan_collapses_inflections() {
        assert_eq!(stem("contractes", Language::Catalan), stem("contracte", Language::Catalan));
        assert_eq!(stem("política",   Language::Catalan), stem("politica",  Language::Catalan));
    }

    #[test]
    fn basque_stems_marrow_examples() {
        assert_eq!(stem("museoak",   Language::Basque), "museo");
        assert_eq!(stem("ikasleak",  Language::Basque), "ikasle");
    }

    #[test]
    fn french_stems_collapse_inflections() {
        let a = stem("annulation", Language::French);
        let b = stem("annuler",    Language::French);
        assert_eq!(a, b);
    }

    #[test]
    fn italian_stems_collapse_inflections() {
        let a = stem("cancellazione", Language::Italian);
        let b = stem("cancellare",    Language::Italian);
        assert_eq!(a, b);
    }

    #[test]
    fn german_stems_collapse_inflections() {
        let a = stem("kündigung", Language::German);
        let b = stem("kündigen",  Language::German);
        assert_eq!(a, b);
    }

    #[test]
    fn dutch_stems_collapse_inflections() {
        let a = stem("opzegging", Language::Dutch);
        let b = stem("opzeggen",  Language::Dutch);
        assert_eq!(a, b);
    }
}
