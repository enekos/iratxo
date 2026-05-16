//! Deterministic, embedding-free semantic similarity.
//!
//! Pipeline: tokenize → drop per-language stop words → Snowball/Basque stem
//! → optional synonym normalization → signed feature hashing into a 256-dim
//! dense vector → cosine similarity.
//!
//! Properties: pure, no embedded model file, wasm-friendly, deterministic.
//! The signal is bag-of-words level — good for "this text is talking about
//! X" matches, not for syntactic reasoning.

use crate::text::{detect_language, is_stopword, stem, stem_cow, tokenize_iter, Language};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

const DIM: usize = 256;

const BUILTIN_SYNONYMS_EN: &str = include_str!("data/synonyms.json");
const BUILTIN_SYNONYMS_ES: &str = include_str!("data/synonyms_es.json");
const BUILTIN_SYNONYMS_CA: &str = include_str!("data/synonyms_ca.json");
const BUILTIN_SYNONYMS_EU: &str = include_str!("data/synonyms_eu.json");
const BUILTIN_SYNONYMS_FR: &str = include_str!("data/synonyms_fr.json");
const BUILTIN_SYNONYMS_IT: &str = include_str!("data/synonyms_it.json");
const BUILTIN_SYNONYMS_DE: &str = include_str!("data/synonyms_de.json");
const BUILTIN_SYNONYMS_NL: &str = include_str!("data/synonyms_nl.json");

fn builtin_source(lang: Language) -> &'static str {
    match lang {
        Language::English => BUILTIN_SYNONYMS_EN,
        Language::Spanish => BUILTIN_SYNONYMS_ES,
        Language::Catalan => BUILTIN_SYNONYMS_CA,
        Language::Basque  => BUILTIN_SYNONYMS_EU,
        Language::French  => BUILTIN_SYNONYMS_FR,
        Language::Italian => BUILTIN_SYNONYMS_IT,
        Language::German  => BUILTIN_SYNONYMS_DE,
        Language::Dutch   => BUILTIN_SYNONYMS_NL,
    }
}

thread_local! {
    /// Per-language cache of the built-in dictionary. Each entry maps
    /// stem(synonym) -> stem(canonical). Built lazily on first use.
    static BUILTIN_BY_LANG: RefCell<FxHashMap<Language, SynonymIndex>> = RefCell::new(FxHashMap::default());
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
///
/// A single synonym stem may map to *multiple* canonicals when the
/// dictionary lists it under several groups (e.g. English "scam" is a
/// synonym of both "phishing" and "fraud"). The embedding contributes to
/// every canonical, so polysemy is preserved instead of one canonical
/// silently winning the alphabetical race.
pub struct SynonymIndex {
    map: FxHashMap<String, Vec<String>>,
}

impl SynonymIndex {
    /// Test-only: peek at the canonical mapping(s) for a stem. Returns an
    /// empty slice if no entry exists. Used by integration tests to assert
    /// that synonym dictionaries actually wire up the mappings users expect.
    #[doc(hidden)]
    pub fn lookup(&self, stem: &str) -> &[String] {
        self.map.get(stem).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Build the index for `lang`. Synonyms and canonical forms are both
    /// stemmed using `lang`'s stemmer.
    ///
    /// Behavioral notes:
    ///   - canonicals are processed in lexical order so identical input
    ///     always produces the same index (the prior HashMap-driven order
    ///     was randomized per process by Rust's per-process hash seed).
    ///   - a synonym stem that appears under multiple canonicals collects
    ///     all of them instead of the last writer winning; the embedding
    ///     hashes each one so the token's signal is distributed across all
    ///     senses.
    ///   - duplicates inside a single stem's canonical list are dropped.
    pub fn from_json_for(src: &str, lang: Language) -> Self {
        let raw: HashMap<String, serde_json::Value> = serde_json::from_str(src).unwrap_or_default();
        let mut entries: Vec<(String, serde_json::Value)> = raw.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut map: FxHashMap<String, Vec<String>> = FxHashMap::default();
        for (canonical, value) in entries {
            if canonical.starts_with('_') { continue; }
            let cstem = stem(&canonical, lang);
            Self::push(&mut map, cstem.clone(), cstem.clone());
            if let Some(arr) = value.as_array() {
                for syn in arr {
                    if let Some(s) = syn.as_str() {
                        Self::push(&mut map, stem(s, lang), cstem.clone());
                    }
                }
            }
        }
        SynonymIndex { map }
    }

    fn push(map: &mut FxHashMap<String, Vec<String>>, key: String, canonical: String) {
        let bucket = map.entry(key).or_default();
        if !bucket.iter().any(|c| c == &canonical) {
            bucket.push(canonical);
        }
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

/// Embed `text` for the given language and optional synonym dictionary.
/// Useful for caching the input embedding when comparing against multiple
/// examples.
pub fn embed_input(text: &str, lang: Language, extra: Option<&SynonymIndex>) -> [f32; DIM] {
    with_builtin(lang, |builtin| embed(text, lang, extra, builtin))
}

/// Same as [`embed_input`] but assumes `text` is already lowercased.
/// Skips the per-token `to_lowercase()` allocation.
pub fn embed_input_lowered(text: &str, lang: Language, extra: Option<&SynonymIndex>) -> [f32; DIM] {
    with_builtin(lang, |builtin| embed_lowered(text, lang, extra, builtin))
}

/// Resolve a stem to one-or-more canonicals. `extra` (rule-supplied) is
/// consulted first; if it has any entry for the stem, those canonicals are
/// used exclusively. Otherwise the built-in dictionary is consulted. If
/// neither produces a hit, the input stem is returned as the sole canonical
/// (so unknown words still produce a hashed bucket).
fn canonicals_for<'a>(
    stem: &'a str,
    extra: Option<&'a SynonymIndex>,
    builtin: &'a SynonymIndex,
) -> CanonIter<'a> {
    if let Some(idx) = extra {
        let v = idx.lookup(stem);
        if !v.is_empty() { return CanonIter::Slice(v); }
    }
    let v = builtin.lookup(stem);
    if !v.is_empty() { CanonIter::Slice(v) } else { CanonIter::Single(stem) }
}

enum CanonIter<'a> { Slice(&'a [String]), Single(&'a str) }

impl<'a> Iterator for CanonIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        match self {
            CanonIter::Slice(s) => {
                if let Some((first, rest)) = s.split_first() {
                    *s = rest;
                    Some(first.as_str())
                } else { None }
            }
            CanonIter::Single(s) => {
                if s.is_empty() { None } else {
                    let out = *s; *s = ""; Some(out)
                }
            }
        }
    }
}

fn hash_into(v: &mut [f32; DIM], s: &str) {
    let h = fnv1a64(s.as_bytes());
    let bucket = (h as usize) % DIM;
    let sign = if (h >> 32) & 1 == 0 { 1.0 } else { -1.0 };
    v[bucket] += sign;
}

/// Per-language input normalization applied before tokenization. Currently
/// only Catalan needs it: the geminated `l·l` digraph (with U+00B7 MIDDLE
/// DOT) would otherwise split into two tokens at the dot — losing the word
/// entirely because the halves don't match the dictionary. Fold to plain
/// `ll` so `cancel·lar` tokenizes as a single word.
fn normalize<'a>(text: &'a str, lang: Language) -> Cow<'a, str> {
    if lang == Language::Catalan && (text.contains('·') || text.contains("L·L")) {
        Cow::Owned(text.replace('·', ""))
    } else {
        Cow::Borrowed(text)
    }
}

fn embed(text: &str, lang: Language, extra: Option<&SynonymIndex>, builtin: &SynonymIndex) -> [f32; DIM] {
    let mut v = [0f32; DIM];
    let text = normalize(text, lang);
    for tok in tokenize_iter(&text) {
        let lower = tok.to_lowercase();
        if is_stopword(&lower, lang) { continue; }
        let stemmed = stem_cow(&lower, lang);
        if stemmed.chars().count() < 2 { continue; }
        for canonical in canonicals_for(&stemmed, extra, builtin) {
            hash_into(&mut v, canonical);
        }
    }
    v
}

/// Longest stopword across all supported languages is 10 chars (English
/// "yourselves" / "themselves"). Using 12 as a safe upper bound lets us
/// skip the binary-search stopword check for the majority of tokens in
/// long documents.
const MAX_STOPWORD_LEN: usize = 12;

fn embed_lowered(text: &str, lang: Language, extra: Option<&SynonymIndex>, builtin: &SynonymIndex) -> [f32; DIM] {
    let mut v = [0f32; DIM];
    let text = normalize(text, lang);
    let text = text.as_ref();
    for tok in tokenize_iter(text) {
        // Fast path: long tokens are never stopwords, skip binary search.
        if tok.len() <= MAX_STOPWORD_LEN && is_stopword(tok, lang) { continue; }
        let stemmed = stem_cow(tok, lang);
        // Fast path for ASCII: len() is much faster than chars().count().
        let len = if stemmed.is_ascii() { stemmed.len() } else { stemmed.chars().count() };
        if len < 2 { continue; }
        for canonical in canonicals_for(&stemmed, extra, builtin) {
            hash_into(&mut v, canonical);
        }
    }
    v
}

/// Cosine similarity between two embeddings.
pub fn cosine(a: &[f32; DIM], b: &[f32; DIM]) -> f32 {
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

pub fn fnv1a64(bytes: &[u8]) -> u64 {
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
    fn french_synonyms_score_high() {
        let s = similarity_lang("je veux annuler mon contrat", "résilier l'accord", Language::French, None);
        assert!(s > 0.4, "got {}", s);
    }

    #[test]
    fn italian_synonyms_score_high() {
        let s = similarity_lang("voglio cancellare il contratto", "annullare l'accordo", Language::Italian, None);
        assert!(s > 0.4, "got {}", s);
    }

    #[test]
    fn german_synonyms_score_high() {
        let s = similarity_lang("ich möchte den vertrag kündigen", "die vereinbarung beenden", Language::German, None);
        assert!(s > 0.4, "got {}", s);
    }

    #[test]
    fn dutch_synonyms_score_high() {
        let s = similarity_lang("ik wil het contract opzeggen", "de overeenkomst annuleren", Language::Dutch, None);
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
