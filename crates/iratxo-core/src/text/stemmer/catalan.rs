//! Light Catalan stemmer.
//!
//! `rust-stemmers` v1.2 does not ship a Catalan algorithm, so we provide a
//! deterministic, hand-rolled light stemmer that strips the highest-frequency
//! nominal, adjectival, and verbal suffixes. It is *not* a full Snowball
//! reimplementation — for the hashing-trick semantic index we just need
//! inflectional variants to collapse to the same form.
//!
//! Procedure:
//!   1. Lowercase + fold diacritics on vowels (`à è é í ï ò ó ú ü` → plain
//!      vowel) so accented forms collapse with un-accented ones. `ç` → `c`.
//!   2. Walk a long-to-short suffix list and strip the first match, provided
//!      the resulting stem is still ≥ 3 characters.
//!   3. Stop. Single pass; idempotent on already-stemmed tokens.

const MIN_STEM: usize = 3;

/// Stem `word` using the light Catalan rules. Returns the stemmed lower-case
/// form. Empty input is returned unchanged.
pub fn stem(word: &str) -> String {
    if word.is_empty() { return String::new(); }
    let folded: String = word.chars().map(fold_char).collect();
    let stripped = strip_suffix(&folded);
    // `qu` before a stripped front vowel is just an orthographic spelling of
    // `c`; collapse so e.g. "polítiqu(es)" → "politic" matches "polític(s)".
    if stripped.ends_with("qu") && stripped.len() > 3 {
        let mut out = String::with_capacity(stripped.len() - 1);
        out.push_str(&stripped[..stripped.len() - 2]);
        out.push('c');
        return out;
    }
    stripped.to_string()
}

fn fold_char(c: char) -> char {
    match c {
        'À' | 'Á' | 'à' | 'á' => 'a',
        'È' | 'É' | 'è' | 'é' => 'e',
        'Í' | 'Ï' | 'í' | 'ï' => 'i',
        'Ò' | 'Ó' | 'ò' | 'ó' => 'o',
        'Ú' | 'Ü' | 'ú' | 'ü' => 'u',
        'Ç' | 'ç' => 'c',
        'Ñ' | 'ñ' => 'n',
        c => c.to_ascii_lowercase(),
    }
}

fn strip_suffix(s: &str) -> &str {
    // Sorted longest-first so we match the most specific suffix.
    // After folding, "ció" → "cio", "tats" stays "tats", etc.
    const SUFFIXES: &[&str] = &[
        // verbal — periphrastic / future / conditional
        "ariem", "arieu", "arien", "ariens",
        "eriem", "erieu", "erien",
        "iriem", "irieu", "irien",
        "assim", "assiu", "essim", "essiu", "issim", "issiu",
        "essin", "issin", "assin",
        "arem", "areu", "aren", "erem", "ereu", "eren",
        "irem", "ireu", "iren",
        "avem", "aveu", "aven", "iem", "ieu", "ien",
        "ariu", "eriu", "iriu",
        "aria", "eria", "iria",
        "ades", "ides", "udes", "uts",
        // nominal / adjectival
        "ments", "ment",
        "acions", "acion", "cions", "cion",
        "encies", "encia", "ancies", "ancia",
        "itats", "itat", "etats", "etat",
        "ismes", "isme", "istes", "ista",
        "dores", "dora", "dors", "dor",
        "ables", "able", "ibles", "ible",
        "osos", "oses", "osa", "os",
        "ada", "ida", "uda",
        // verbal participles / continuous
        "ava", "aves",
        "ant", "ent", "int",
        "are", "ere", "ire",
        "ats", "ets", "its",
        "at", "et", "it", "ut",
        "ar", "er", "ir",
        // articles, plurals, gender
        "es",
        "a", "e", "o", "s",
    ];

    for suf in SUFFIXES {
        if s.len() > suf.len() + (MIN_STEM - 1) && s.ends_with(suf) {
            let cut = s.len() - suf.len();
            if s.is_char_boundary(cut) {
                return &s[..cut];
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_plurals() {
        assert_eq!(stem("contractes"), stem("contracte"));
        assert_eq!(stem("usuaris"), stem("usuari"));
    }

    #[test]
    fn collapses_verbal_inflections() {
        let a = stem("cancellar");
        let b = stem("cancelles");
        let c = stem("cancellava");
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn handles_diacritics() {
        let a = stem("polítiques");
        let b = stem("política");
        let c = stem("polítics");
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn protects_short_words() {
        assert_eq!(stem("el"), "el");
        assert_eq!(stem("amb"), "amb");
    }

    #[test]
    fn idempotent() {
        let once = stem("cancellava");
        let twice = stem(&once);
        assert_eq!(once, twice);
    }
}
