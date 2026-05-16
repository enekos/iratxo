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
        Language::French  => FR,
        Language::Italian => IT,
        Language::German  => DE,
        Language::Dutch   => NL,
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
const FR: &[&str] = &[
    "ai", "aie", "aient", "aies", "ainsi", "ait", "alors", "après", "as", "au", "aucun", "aussi",
    "aux", "avant", "avec", "avoir", "ayant", "car", "ce", "ceci", "cela", "ces", "cet", "cette",
    "ceux", "chaque", "comme", "comment", "d", "dans", "de", "dehors", "des", "du", "elle", "elles",
    "en", "encore", "es", "est", "et", "eu", "eut", "fais", "faisait", "faisant", "fait", "faut",
    "fois", "fut", "hors", "il", "ils", "j", "je", "l", "la", "le", "les", "leur",
    "leurs", "lui", "là", "m", "ma", "mais", "me", "mes", "moi", "mon", "même", "n",
    "ne", "ni", "non", "nos", "notre", "nous", "on", "ont", "ou", "où", "par", "parce",
    "pas", "pendant", "peu", "plus", "plutôt", "pour", "pourquoi", "qu", "quand", "que", "quel", "quelle",
    "quelles", "quels", "qui", "quoi", "s", "sa", "sans", "se", "ses", "si", "sien", "soi",
    "soit", "son", "sont", "sous", "sur", "ta", "te", "tes", "toi", "ton", "tous", "tout",
    "toute", "toutes", "très", "tu", "un", "une", "vos", "votre", "vous", "y", "à", "ça",
    "étaient", "était", "été", "être",
];
const IT: &[&str] = &[
    "a", "abbiamo", "ad", "agli", "ai", "al", "alcuni", "alla", "alle", "allo", "altre", "altri",
    "altro", "anche", "ancora", "avere", "aveva", "avevano", "c", "che", "chi", "ci", "co", "come",
    "con", "contro", "cui", "d", "da", "dagli", "dai", "dal", "dalla", "dalle", "dallo", "degli",
    "dei", "del", "della", "delle", "dello", "di", "dopo", "dove", "due", "e", "ed", "egli",
    "ella", "era", "erano", "essere", "fa", "fra", "gli", "ha", "hai", "hanno", "ho", "i",
    "il", "in", "io", "l", "la", "le", "lei", "li", "lo", "loro", "lui", "ma",
    "me", "mentre", "mi", "mia", "mie", "miei", "mio", "ne", "negli", "nei", "nel", "nella",
    "nelle", "nello", "no", "noi", "non", "nostra", "nostre", "nostri", "nostro", "o", "ogni", "per",
    "perché", "però", "più", "poi", "quale", "quali", "quando", "quanto", "quei", "quel", "quella", "quelle",
    "quelli", "quello", "questa", "queste", "questi", "questo", "se", "sei", "senza", "siamo", "siete", "sono",
    "sopra", "sotto", "su", "sua", "sue", "sugli", "sui", "sul", "sulla", "sulle", "sullo", "suo",
    "suoi", "te", "ti", "tra", "tu", "tua", "tue", "tuo", "tuoi", "tutta", "tutte", "tutti",
    "tutto", "un", "una", "uno", "voi", "vostra", "vostre", "vostri", "vostro", "è",
];
const DE: &[&str] = &[
    "aber", "alle", "allem", "allen", "aller", "alles", "als", "also", "am", "an", "ander", "anders",
    "auch", "auf", "aus", "bei", "bin", "bis", "bist", "da", "damit", "dann", "das", "dass",
    "dazu", "daß", "dein", "deine", "dem", "den", "der", "des", "deshalb", "dessen", "die", "dies",
    "dieser", "dieses", "doch", "dort", "du", "durch", "ein", "eine", "einem", "einen", "einer", "eines",
    "er", "es", "etwa", "euch", "euer", "eure", "für", "gegen", "habe", "haben", "hat", "hatte",
    "hatten", "hier", "hinter", "ich", "ihm", "ihn", "ihnen", "ihr", "ihre", "im", "in", "indem",
    "ist", "ja", "jede", "jeden", "jedoch", "kann", "kein", "keine", "können", "machen", "man", "mein",
    "meine", "mit", "muss", "mußt", "müssen", "müßt", "nach", "nein", "nicht", "nichts", "noch", "nun",
    "nur", "ob", "oder", "ohne", "schon", "sehr", "sein", "seine", "seinem", "seinen", "seiner", "seines",
    "sich", "sie", "sind", "so", "sollte", "sondern", "sonst", "um", "und", "uns", "unser", "unsere",
    "unter", "vom", "von", "vor", "war", "waren", "was", "weil", "welche", "wem", "wen", "wenn",
    "wer", "werde", "werden", "wie", "wieder", "will", "wir", "wird", "wo", "während", "wäre", "würde",
    "würden", "zu", "zum", "zur", "zwar", "zwischen", "über",
];
const NL: &[&str] = &[
    "aan", "af", "al", "alle", "alleen", "als", "andere", "anders", "ben", "bij", "binnen", "buiten",
    "daar", "dan", "dat", "de", "deze", "die", "dit", "door", "dus", "een", "eens", "elk",
    "elke", "en", "enkele", "er", "even", "ga", "gaan", "geen", "haar", "had", "hadden", "heb",
    "hebben", "heeft", "hem", "hen", "het", "hier", "hij", "hoe", "hoeveel", "hun", "iemand", "iets",
    "ik", "in", "is", "ja", "je", "jij", "jou", "jouw", "jullie", "kan", "kon", "kunnen",
    "maar", "mag", "me", "meer", "meest", "men", "met", "mij", "mijn", "moet", "moeten", "na",
    "naar", "nee", "niemand", "niet", "niets", "nog", "nu", "of", "om", "omdat", "ons", "onze",
    "ook", "op", "over", "te", "tegen", "toen", "tot", "tussen", "u", "uit", "uw", "van",
    "veel", "voor", "waar", "waarom", "wanneer", "want", "waren", "was", "wat", "we", "wel", "welk",
    "welke", "werd", "werden", "wie", "wij", "wilde", "willen", "worden", "wordt", "ze", "zelf", "zich",
    "zij", "zijn", "zo", "zonder", "zou", "zouden",
];

#[cfg(test)]
mod tests {
    use super::*;

    // Per-language stopword coverage: ~10 function-word positives must be
    // stopwords; a representative set of content words must not. Content-word
    // negatives are picked from the same compliance/legal vocabulary the
    // semantic dictionaries cover, so any drift in stopword lists that wipes
    // out real signal trips a test.

    fn check(lang: Language, positives: &[&str], negatives: &[&str]) {
        for w in positives {
            assert!(is_stopword(w, lang), "{:?}: expected {:?} to be a stopword", lang, w);
        }
        for w in negatives {
            assert!(!is_stopword(w, lang), "{:?}: expected {:?} NOT to be a stopword", lang, w);
        }
    }

    #[test]
    fn english() {
        check(Language::English,
            &["the", "a", "an", "and", "or", "of", "to", "in", "for", "with", "is", "are", "this", "that"],
            &["dog", "contract", "refund", "policy", "support", "delete", "password"]);
    }

    #[test]
    fn spanish() {
        check(Language::Spanish,
            &["el", "la", "los", "las", "un", "una", "de", "que", "y", "en", "por", "para", "con", "es"],
            &["perro", "contrato", "reembolso", "política", "soporte", "borrar", "contraseña"]);
    }

    #[test]
    fn catalan() {
        check(Language::Catalan,
            &["el", "la", "els", "les", "un", "una", "amb", "de", "que", "i", "en", "per", "no", "ja"],
            &["gos", "contracte", "reemborsament", "política", "suport", "esborrar", "contrasenya"]);
    }

    #[test]
    fn basque() {
        check(Language::Basque,
            &["eta", "da", "du", "edo", "bai", "ez", "izan", "nire", "zuen", "haien"],
            &["etxea", "kontratu", "diru", "politika", "laguntza", "ezabatu", "pasahitz"]);
    }

    #[test]
    fn french() {
        check(Language::French,
            &["le", "la", "les", "un", "une", "des", "de", "du", "et", "que", "qui", "à", "être", "avoir", "ce"],
            &["chien", "contrat", "remboursement", "politique", "support", "supprimer", "passe"]);
    }

    #[test]
    fn italian() {
        check(Language::Italian,
            &["il", "la", "lo", "i", "gli", "le", "un", "una", "di", "che", "e", "in", "per", "è", "perché"],
            &["cane", "contratto", "rimborso", "politica", "supporto", "eliminare", "password"]);
    }

    #[test]
    fn german() {
        check(Language::German,
            &["der", "die", "das", "ein", "eine", "und", "oder", "in", "für", "mit", "ist", "sind", "nicht", "auf"],
            &["hund", "vertrag", "rückerstattung", "richtlinie", "unterstützung", "löschen", "passwort"]);
    }

    #[test]
    fn dutch() {
        check(Language::Dutch,
            &["de", "het", "een", "en", "of", "van", "in", "voor", "met", "is", "zijn", "niet", "dat", "die"],
            &["hond", "contract", "terugbetaling", "beleid", "ondersteuning", "verwijderen", "wachtwoord"]);
    }

    #[test]
    fn longest_stopword_within_fast_path_budget() {
        // semantic::MAX_STOPWORD_LEN = 12 bytes is the threshold above which
        // the embed_lowered() fast path skips the binary_search. If any list
        // gains a longer stopword the constant must move in lock-step.
        const BUDGET: usize = 12;
        for list in [EN, ES, CA, EU, FR, IT, DE, NL] {
            for w in list {
                assert!(w.len() <= BUDGET, "stopword {:?} exceeds MAX_STOPWORD_LEN={}", w, BUDGET);
            }
        }
    }

    #[test]
    fn lists_are_sorted() {
        for w in [
            EN.windows(2), ES.windows(2), CA.windows(2), EU.windows(2),
            FR.windows(2), IT.windows(2), DE.windows(2), NL.windows(2),
        ] {
            for pair in w {
                assert!(pair[0] < pair[1], "stop word list out of order near {:?}", pair);
            }
        }
    }
}
