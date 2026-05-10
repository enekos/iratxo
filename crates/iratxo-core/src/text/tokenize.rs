//! Unicode-aware tokenizer. Splits on any character that is not a letter or
//! digit (Unicode-class-aware), lowercases each token. Mirrors marrow's
//! `wordSplitter = regexp.MustCompile("[^\\p{L}\\p{N}]+")`.

/// Zero-allocation token iterator. Yields borrowed slices.
pub fn tokenize_iter(text: &str) -> impl Iterator<Item = &str> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
}

/// Token iterator that yields byte-offset pairs `(start, end)` instead of
/// slices. Useful for caching token positions without lifetime issues.
pub fn tokenize_offsets(text: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(move |s| {
            let start = s.as_ptr() as usize - text.as_ptr() as usize;
            (start, start + s.len())
        })
}

pub fn tokenize(text: &str) -> Vec<String> {
    tokenize_iter(text)
        .map(|s| s.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_punctuation_and_keeps_unicode() {
        let toks = tokenize("¿Hola, niño? Aquí está el dueño.");
        assert_eq!(toks, vec!["hola", "niño", "aquí", "está", "el", "dueño"]);
    }

    #[test]
    fn handles_basque_chars() {
        let toks = tokenize("Etxean nago—joango naiz!");
        assert_eq!(toks, vec!["etxean", "nago", "joango", "naiz"]);
    }
}
