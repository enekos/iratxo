//! Unicode-aware tokenizer. Splits on any character that is not a letter or
//! digit (Unicode-class-aware), lowercases each token. Mirrors marrow's
//! `wordSplitter = regexp.MustCompile("[^\\p{L}\\p{N}]+")`.

pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
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
