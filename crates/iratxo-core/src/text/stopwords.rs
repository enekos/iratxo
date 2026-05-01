//! Per-language stop-word sets. Ported from marrow stopwords.go.
//! Stored as sorted slices for branch-free `binary_search`.

use crate::text::lang::Language;

pub fn is_stopword(word: &str, lang: Language) -> bool {
    let list = match lang {
        Language::English => EN,
        Language::Spanish => ES,
        Language::Basque  => EU,
    };
    list.binary_search(&word).is_ok()
}

const EN: &[&str] = &[
    "a", "above", "after", "an", "and", "any", "are", "as", "at", "be", "because", "been",
    "before", "being", "below", "between", "both", "but", "by", "can", "could", "did", "do",
    "does", "during", "each", "for", "from", "had", "has", "have", "he", "her", "here", "him",
    "his", "how", "i", "if", "in", "into", "is", "it", "its", "just", "like", "may", "me",
    "might", "more", "most", "must", "my", "no", "not", "now", "of", "on", "once", "only", "or",
    "other", "ought", "our", "own", "same", "she", "should", "so", "some", "still", "such",
    "than", "that", "the", "their", "them", "then", "there", "these", "they", "this", "those",
    "though", "through", "to", "too", "under", "up", "us", "very", "was", "we", "were", "what",
    "whatever", "when", "where", "whether", "which", "while", "who", "whom", "whose", "why",
    "will", "with", "would", "yes", "yet", "you", "your",
];
const ES: &[&str] = &[
    "a", "al", "ante", "aún", "bajo", "como", "con", "cual", "cuales", "cuando", "cuanta",
    "cuanto", "cuya", "cuyas", "cuyo", "cuyos", "de", "del", "desde", "donde", "durante", "el",
    "en", "entre", "es", "está", "están", "excepto", "fue", "fueron", "ha", "había", "hacia",
    "han", "hasta", "la", "las", "le", "les", "lo", "los", "me", "mediante", "muy", "más", "no",
    "nos", "o", "os", "para", "pero", "por", "que", "quien", "salvo", "se", "según", "sin",
    "sino", "sobre", "son", "te", "tras", "un", "una", "unas", "unos", "y",
];
const EU: &[&str] = &[
    "ala", "bada", "bai", "baina", "baino", "beraz", "berori", "bestela", "beti", "da", "dago",
    "daude", "ditu", "du", "dute", "edo", "egin", "ere", "eta", "ez", "gainera", "gu", "gutxi",
    "guzti", "haiek", "haien", "han", "haren", "hau", "hemen", "honela", "hori", "hortxe",
    "hura", "inoiz", "izan", "ni", "nire", "noiz", "nola", "nor", "noren", "nori", "nork",
    "oso", "zenbat", "zer", "zergatik", "zu", "zuek", "zuen", "zure",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english() {
        assert!(is_stopword("the", Language::English));
        assert!(!is_stopword("dog", Language::English));
    }

    #[test]
    fn spanish() {
        assert!(is_stopword("el", Language::Spanish));
        assert!(!is_stopword("perro", Language::Spanish));
    }

    #[test]
    fn basque() {
        assert!(is_stopword("eta", Language::Basque));
        assert!(!is_stopword("etxea", Language::Basque));
    }

    /// All const slices must be sorted, otherwise binary_search returns
    /// false negatives.
    #[test]
    fn lists_are_sorted() {
        for w in [EN.windows(2), ES.windows(2), EU.windows(2)] {
            for pair in w {
                assert!(pair[0] < pair[1], "stop word list out of order near {:?}", pair);
            }
        }
    }
}
