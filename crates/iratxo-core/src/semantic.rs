//! Deterministic, embedding-free semantic similarity.
//!
//! Pipeline: tokenize → drop per-language stop words → Snowball/Basque stem
//! → optional synonym normalization → signed feature hashing into a 256-dim
//! dense vector → cosine similarity.
//!
//! Properties: pure, no embedded model file, wasm-friendly, deterministic.
//! The signal is bag-of-words level — good for "this text is talking about
//! X" matches, not for syntactic reasoning.

use crate::text::{detect_language, is_stopword, stem, tokenize, Language};
use std::cell::RefCell;
use std::collections::HashMap;

const DIM: usize = 256;

const BUILTIN_SYNONYMS_EN: &str = include_str!("data/synonyms.json");
const BUILTIN_SYNONYMS_ES: &str = include_str!("data/synonyms_es.json");
const BUILTIN_SYNONYMS_CA: &str = include_str!("data/synonyms_ca.json");
const BUILTIN_SYNONYMS_EU: &str = include_str!("data/synonyms_eu.json");

fn builtin_source(lang: Language) -> &'static str {
    match lang {
        Language::English => BUILTIN_SYNONYMS_EN,
        Language::Spanish => BUILTIN_SYNONYMS_ES,
        Language::Catalan => BUILTIN_SYNONYMS_CA,
        Language::Basque  => BUILTIN_SYNONYMS_EU,
    }
}

thread_local! {
    /// Per-language cache of the built-in dictionary. Each entry maps
    /// stem(synonym) -> stem(canonical). Built lazily on first use.
    static BUILTIN_BY_LANG: RefCell<HashMap<Language, SynonymIndex>> = RefCell::new(HashMap::new());
}

fn with_builtin<R>(lang: Language, f: impl FnOnce(&SynonymIndex) -> R) -> R {
    BUILTIN_BY_LANG.with(|cell| {
        let mut map = cell.borrow_mut();
        let idx = map.entry(lang).or_insert_with(|| {
            SynonymIndex::from_json_for(builtin_source(lang), lang)
        });
        f(idx)
    })
}

/// Synonyms collapsed to canonical form, with both keys and values pre-stemmed
/// for the chosen language so lookups can use stems directly.
pub struct SynonymIndex {
    map: HashMap<String, String>,
}

impl SynonymIndex {
    /// Build the index for `lang`. Synonyms and canonical forms are both
    /// stemmed using `lang`'s stemmer.
    pub fn from_json_for(src: &str, lang: Language) -> Self {
        let raw: HashMap<String, serde_json::Value> = serde_json::from_str(src).unwrap_or_default();
        let mut map = HashMap::new();
        for (canonical, value) in raw {
            if canonical.starts_with('_') { continue; }
            let cstem = stem(&canonical, lang);
            map.insert(cstem.clone(), cstem.clone());
            if let Some(arr) = value.as_array() {
                for syn in arr {
                    if let Some(s) = syn.as_str() {
                        map.insert(stem(s, lang), cstem.clone());
                    }
                }
            }
        }
        SynonymIndex { map }
    }
}

/// Cosine similarity in [-1.0, 1.0] between `input` and `example`. Language
/// is auto-detected from the input.
pub fn similarity(input: &str, example: &str) -> f32 {
    let lang = detect_language(input);
    similarity_lang(input, example, lang, None)
}

/// Same as [`similarity`] but with an explicit language and an optional
/// rule-supplied dictionary (looked up before the built-in one).
pub fn similarity_lang(input: &str, example: &str, lang: Language, extra: Option<&SynonymIndex>) -> f32 {
    with_builtin(lang, |builtin| {
        let a = embed(input, lang, extra, builtin);
        let b = embed(example, lang, extra, builtin);
        cosine(&a, &b)
    })
}

fn canonicalize_stem(stem: &str, extra: Option<&SynonymIndex>, builtin: &SynonymIndex) -> String {
    if let Some(idx) = extra {
        if let Some(c) = idx.map.get(stem) { return c.clone(); }
    }
    builtin.map.get(stem).cloned().unwrap_or_else(|| stem.to_string())
}

fn embed(text: &str, lang: Language, extra: Option<&SynonymIndex>, builtin: &SynonymIndex) -> [f32; DIM] {
    let mut v = [0f32; DIM];
    for tok in tokenize(text) {
        if is_stopword(&tok, lang) { continue; }
        let stemmed = stem(&tok, lang);
        if stemmed.chars().count() < 2 { continue; }
        let canonical = canonicalize_stem(&stemmed, extra, builtin);
        let h = fnv1a64(canonical.as_bytes());
        let bucket = (h as usize) % DIM;
        let sign = if (h >> 32) & 1 == 0 { 1.0 } else { -1.0 };
        v[bucket] += sign;
    }
    v
}

fn cosine(a: &[f32; DIM], b: &[f32; DIM]) -> f32 {
    let mut dot = 0f32;
    let mut na = 0f32;
    let mut nb = 0f32;
    for i in 0..DIM {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 { return 0.0; }
    dot / (na.sqrt() * nb.sqrt())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_synonyms_score_high() {
        let s = similarity("cancel the agreement", "terminate the contract");
        assert!(s > 0.6, "got {}", s);
    }

    #[test]
    fn english_unrelated_scores_low() {
        let s = similarity("the cat sat on the mat", "terminate the contract");
        assert!(s < 0.3, "got {}", s);
    }

    #[test]
    fn english_stemming_collapses_inflections() {
        let s = similarity("cancelling our agreements", "cancel agreement");
        assert!(s > 0.8, "got {}", s);
    }

    #[test]
    fn spanish_inflections_collapse() {
        // "corriendo" and "correr" should share the stem "corr".
        let s = similarity_lang("estoy corriendo", "voy a correr", Language::Spanish, None);
        assert!(s > 0.4, "got {}", s);
    }

    #[test]
    fn basque_inflections_collapse() {
        // "museoak" → "museo", "museoan" → "museo" via Basque stemmer.
        let s = similarity_lang("museoak handiak", "museoan nago", Language::Basque, None);
        assert!(s > 0.4, "got {}", s);
    }

    #[test]
    fn auto_detect_routes_to_correct_stemmer() {
        // Auto-detected as Basque thanks to "tx"/"tz" plus stopword "eta".
        // After aggressive Basque stemming the texts share just one bucket,
        // so scores are modest — still well above unrelated text.
        let s = similarity("Etxean nago eta liburuak irakurtzen ari naiz",
                           "etxean liburua dut");
        let unrelated = similarity("hello there friend",
                                   "Etxean nago eta liburuak");
        assert!(s > 0.2, "Basque-Basque got {}", s);
        assert!(s > unrelated, "Basque-Basque ({}) should beat unrelated ({})", s, unrelated);
    }
}
