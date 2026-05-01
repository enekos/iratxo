//! Language-aware stemmer dispatch.
//!
//! - English / Spanish: delegated to `rust-stemmers`, which implements the
//!   Snowball algorithms (the same family marrow uses).
//! - Catalan: hand-rolled light stemmer (rust-stemmers does not ship Catalan).
//! - Basque: hand-port of marrow's stemmer (no upstream Rust crate exists).

mod basque;
mod snowball_word;

use crate::text::lang::Language;
use rust_stemmers::{Algorithm, Stemmer};
use std::sync::OnceLock;

static EN_STEMMER: OnceLock<Stemmer> = OnceLock::new();
static ES_STEMMER: OnceLock<Stemmer> = OnceLock::new();

pub fn stem(word: &str, lang: Language) -> String {
    if word.is_empty() { return String::new(); }
    match lang {
        Language::English => EN_STEMMER.get_or_init(|| Stemmer::create(Algorithm::English)).stem(word).into_owned(),
        Language::Spanish => ES_STEMMER.get_or_init(|| Stemmer::create(Algorithm::Spanish)).stem(word).into_owned(),
        Language::Basque  => basque::stem(word),
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
    fn basque_stems_marrow_examples() {
        assert_eq!(stem("museoak",   Language::Basque), "museo");
        assert_eq!(stem("ikasleak",  Language::Basque), "ikasle");
    }
}
