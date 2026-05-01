//! Basque stemmer. Direct port of marrow's
//! `internal/stemmer/basque/{stem,suffixes}.go`.
//!
//! High-level shape: mark R1/R2/RV regions, then run three suffix-stripping
//! passes — verbal (`aditzak`), nominal (`izenak`), and adjectival
//! (`adjetiboak`). The first two loop until they no longer match; the third
//! runs once. Each table is consulted in order, so longer/more-specific
//! suffixes must come first — that's how marrow's tables are arranged and we
//! preserve that order.

mod suffix_table {
    include!("basque_suffixes.rs");
}

use super::snowball_word::{vnv_suffix, SnowballWord};
use suffix_table::{ADITZAK_SUFFIXES, ADJETIBOAK_SUFFIXES, IZENAK_SUFFIXES};

pub fn stem(word: &str) -> String {
    let lowered = word.trim().to_lowercase();
    if lowered.chars().count() <= 2 {
        return lowered;
    }

    let mut w = SnowballWord::new(&lowered);
    mark_regions(&mut w);

    while aditzak(&mut w) {}
    while izenak(&mut w) {}
    adjetiboak(&mut w);

    w.to_string()
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// Marrow's `markRegions`. Sets pV (RV) via either of two cursor walks, then
/// computes p1/p2 with romance.VnvSuffix.
fn mark_regions(w: &mut SnowballWord) {
    let limit = w.rs.len();
    let mut pv = limit;

    let mut cursor = 0usize;
    if try_pv_option_a(&w.rs, limit, &mut cursor) {
        pv = cursor;
    } else {
        cursor = 0;
        if try_pv_option_b(&w.rs, limit, &mut cursor) {
            pv = cursor;
        }
    }

    let p1 = vnv_suffix(&w.rs, is_vowel, 0);
    let p2 = vnv_suffix(&w.rs, is_vowel, p1);

    w.r1_start = p1;
    w.r2_start = p2;
    w.rv_start = pv;
}

fn try_pv_option_a(rs: &[char], limit: usize, cursor: &mut usize) -> bool {
    let mut c = *cursor;
    if c >= limit || !is_vowel(rs[c]) { return false; }
    c += 1;

    let v3 = c;
    if c < limit && !is_vowel(rs[c]) {
        c += 1;
        while c < limit && !is_vowel(rs[c]) { c += 1; }
        if c < limit {
            *cursor = c + 1;
            return true;
        }
    }
    c = v3;
    if c < limit && is_vowel(rs[c]) {
        c += 1;
        while c < limit && is_vowel(rs[c]) { c += 1; }
        if c < limit {
            *cursor = c + 1;
            return true;
        }
    }
    false
}

fn try_pv_option_b(rs: &[char], limit: usize, cursor: &mut usize) -> bool {
    let mut c = *cursor;
    if c >= limit || is_vowel(rs[c]) { return false; }
    c += 1;

    let v4 = c;
    if c < limit && !is_vowel(rs[c]) {
        c += 1;
        while c < limit && !is_vowel(rs[c]) { c += 1; }
        if c < limit {
            *cursor = c + 1;
            return true;
        }
    }
    c = v4;
    if c < limit && is_vowel(rs[c]) {
        c += 1;
        if c < limit {
            *cursor = c + 1;
            return true;
        }
    }
    false
}

type Row = (&'static [char], i8);

fn find_suffix(w: &SnowballWord, table: &[Row]) -> Option<Row> {
    table.iter().find(|(s, _)| w.has_suffix(s)).copied()
}

fn aditzak(w: &mut SnowballWord) -> bool {
    let Some((suffix, action)) = find_suffix(w, ADITZAK_SUFFIXES) else { return false; };
    let suffix_start = w.rs.len() - suffix.len();
    match action {
        1 => {
            if suffix_start < w.rv_start { return false; }
            w.remove_last_n(suffix.len());
            true
        }
        2 => {
            if suffix_start < w.r2_start { return false; }
            w.remove_last_n(suffix.len());
            true
        }
        _ => false, // -1 marks suffixes that block further stripping (e.g. "arabera").
    }
}

fn izenak(w: &mut SnowballWord) -> bool {
    let Some((suffix, action)) = find_suffix(w, IZENAK_SUFFIXES) else { return false; };
    let suffix_start = w.rs.len() - suffix.len();
    match action {
        1 => {
            if suffix_start < w.rv_start { return false; }
            w.remove_last_n(suffix.len());
            true
        }
        2 => {
            if suffix_start < w.r2_start { return false; }
            w.remove_last_n(suffix.len());
            true
        }
        3 => { w.replace_suffix(suffix, &['j','o','k']); true }
        4 => {
            if suffix_start < w.r1_start { return false; }
            w.remove_last_n(suffix.len());
            true
        }
        5 => { w.replace_suffix(suffix, &['t','r','a']); true }
        6 => { w.replace_suffix(suffix, &['m','i','n','u','t','u']); true }
        _ => false,
    }
}

fn adjetiboak(w: &mut SnowballWord) -> bool {
    let Some((suffix, action)) = find_suffix(w, ADJETIBOAK_SUFFIXES) else { return false; };
    let suffix_start = w.rs.len() - suffix.len();
    match action {
        1 => {
            if suffix_start < w.rv_start { return false; }
            w.remove_last_n(suffix.len());
            true
        }
        2 => { w.replace_suffix(suffix, &['z']); true }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_words_pass_through() {
        assert_eq!(stem("a"),  "a");
        assert_eq!(stem("ez"), "ez");
    }

    #[test]
    fn known_examples() {
        // Outputs verified against marrow's reference implementation.
        assert_eq!(stem("museoak"),  "museo");
        assert_eq!(stem("ikasleak"), "ikasle");
    }
}
