//! Per-language stop-word sets. Stored as sorted slices for branch-free
//! `binary_search`. Lists were curated from open-source projects (snowball,
//! stopwords-iso) and trimmed to closed-class function words — content-bearing
//! words are kept in to preserve signal in the semantic index.

use crate::text::lang::Language;

pub fn is_stopword(word: &str, lang: Language) -> bool {
    let list = match lang {
        Language::English => EN,
        Language::Spanish => ES,
        Language::Catalan => CA,
        Language::Basque  => EU,
    };
    list.binary_search(&word).is_ok()
}

const EN: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and", "any", "are",
    "aren", "as", "at", "be", "because", "been", "before", "being", "below", "between", "both",
    "but", "by", "can", "could", "couldn", "did", "didn", "do", "does", "doesn", "doing", "don",
    "down", "during", "each", "few", "for", "from", "further", "had", "hadn", "has", "hasn",
    "have", "haven", "having", "he", "her", "here", "hers", "herself", "him", "himself", "his",
    "how", "i", "if", "in", "into", "is", "isn", "it", "its", "itself", "just", "like", "may",
    "me", "might", "more", "most", "must", "my", "myself", "no", "nor", "not", "now", "of",
    "off", "on", "once", "only", "or", "other", "ought", "our", "ours", "ourselves", "out",
    "over", "own", "same", "she", "should", "shouldn", "so", "some", "still", "such", "than",
    "that", "the", "their", "theirs", "them", "themselves", "then", "there", "these", "they",
    "this", "those", "though", "through", "to", "too", "under", "until", "up", "us", "very",
    "was", "wasn", "we", "were", "weren", "what", "whatever", "when", "where", "whether",
    "which", "while", "who", "whom", "whose", "why", "will", "with", "won", "would", "wouldn",
    "yes", "yet", "you", "your", "yours", "yourself", "yourselves",
];
const ES: &[&str] = &[
    "a", "al", "algo", "alguna", "algunas", "alguno", "algunos", "algún", "ante", "antes",
    "aquel", "aquella", "aquellas", "aquellos", "aquí", "así", "aún", "bajo", "bien", "cada",
    "como", "con", "contra", "cual", "cuales", "cuando", "cuanta", "cuanto", "cuya", "cuyas",
    "cuyo", "cuyos", "cómo", "de", "del", "desde", "donde", "durante", "e", "el", "ella",
    "ellas", "ello", "ellos", "en", "entre", "era", "eran", "eres", "es", "esa", "esas",
    "ese", "eso", "esos", "esta", "estamos", "estas", "este", "esto", "estos", "estoy",
    "está", "están", "excepto", "fue", "fueron", "ha", "había", "hace", "hacia", "han",
    "hasta", "hay", "la", "las", "le", "les", "lo", "los", "luego", "me", "mediante",
    "mejor", "mi", "mientras", "mis", "mismo", "mucho", "muy", "más", "nada", "ni", "ningún",
    "no", "nos", "nosotros", "nuestra", "nuestras", "nuestro", "nuestros", "nunca", "o", "os",
    "otra", "otras", "otro", "otros", "para", "pero", "poco", "por", "porque", "pronto",
    "pues", "que", "quien", "qué", "salvo", "se", "según", "ser", "si", "siempre", "sin",
    "sino", "sobre", "son", "soy", "su", "sus", "sí", "también", "tampoco", "tan", "tanto",
    "te", "tenemos", "tener", "tengo", "tiene", "tienen", "todo", "todos", "tras", "tu",
    "tus", "un", "una", "unas", "uno", "unos", "y", "ya", "yo",
];
const CA: &[&str] = &[
    "a", "abans", "ací", "així", "al", "als", "amb", "ambdós", "ans", "aquell", "aquella",
    "aquelles", "aquells", "aquest", "aquesta", "aquestes", "aquests", "ara", "açò", "bé",
    "cada", "cap", "com", "contra", "d", "dalt", "darrer", "de", "del", "dels", "des",
    "després", "doncs", "durant", "e", "el", "ell", "ella", "elles", "ells", "els", "em",
    "en", "encara", "entre", "era", "eren", "es", "essent", "estan", "estar", "està", "et",
    "ets", "fa", "fer", "feu", "fins", "fou", "ha", "haver", "havien", "hem", "hi", "ho",
    "i", "ja", "jo", "l", "la", "les", "li", "llavors", "lo", "los", "m", "mai", "massa",
    "meu", "meus", "meva", "meves", "mi", "molt", "molta", "moltes", "molts", "més", "n",
    "ni", "ningú", "no", "nosaltres", "nostra", "nostre", "nostres", "o", "on", "pas", "pel",
    "pels", "per", "perquè", "però", "poc", "pocs", "que", "qui", "quin", "quina", "quines",
    "quins", "què", "s", "sa", "se", "segons", "sempre", "sense", "ser", "ses", "seu",
    "seus", "seva", "seves", "si", "sobre", "som", "sota", "sou", "sí", "són", "ta", "tal",
    "també", "tant", "tanta", "tantes", "tants", "te", "tenia", "teniu", "tens", "teu",
    "teus", "teva", "teves", "ton", "tot", "tota", "totes", "tots", "tu", "u", "un", "una",
    "unes", "uns", "us", "va", "vaig", "vam", "van", "vau", "vosaltres", "vostra", "vostre",
    "vostres", "y", "ya",
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
    fn catalan() {
        assert!(is_stopword("el", Language::Catalan));
        assert!(is_stopword("amb", Language::Catalan));
        assert!(!is_stopword("gos", Language::Catalan));
    }

    #[test]
    fn basque() {
        assert!(is_stopword("eta", Language::Basque));
        assert!(!is_stopword("etxea", Language::Basque));
    }

    #[test]
    fn lists_are_sorted() {
        for w in [EN.windows(2), ES.windows(2), CA.windows(2), EU.windows(2)] {
            for pair in w {
                assert!(pair[0] < pair[1], "stop word list out of order near {:?}", pair);
            }
        }
    }
}
