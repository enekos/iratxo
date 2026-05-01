//! Heuristic language detection for English / Spanish / Catalan / Basque.
//!
//! Character-level signals (e.g. `ñ`, `ç`, accented vowels, `tx`/`tz` digraphs,
//! Catalan `l·l` and trailing `-ció`) plus per-language closed word lists.
//! Tie-breaks favor English.

use crate::text::tokenize::tokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    English,
    Spanish,
    Catalan,
    Basque,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::Catalan => "ca",
            Language::Basque  => "eu",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" | "english" => Some(Language::English),
            "es" | "spanish" => Some(Language::Spanish),
            "ca" | "catalan" | "valencian" | "val" => Some(Language::Catalan),
            "eu" | "basque"  => Some(Language::Basque),
            _ => None,
        }
    }

    /// Iterate every supported language. Stable order; useful in tests and tooling.
    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Spanish, Language::Catalan, Language::Basque]
    }
}

pub fn detect_language(text: &str) -> Language {
    let lower = text.to_lowercase();
    let mut s_en = 0i32;
    let mut s_es = 0i32;
    let mut s_ca = 0i32;
    let mut s_eu = 0i32;

    for c in lower.chars() {
        match c {
            'ñ' | '¿' | '¡' => s_es += 10,
            'ç' => s_ca += 10,
            'à' | 'è' | 'ò' => s_ca += 6,
            'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü' => { s_es += 3; s_ca += 1; }
            'ï' => s_ca += 3,
            _ => {}
        }
    }
    if lower.contains("l·l") || lower.contains("l.l") { s_ca += 12; }
    if lower.contains("ny") { s_ca += 2; }
    if lower.contains("tx") { s_eu += 5; s_ca += 1; }
    if lower.contains("tz") { s_eu += 3; }
    if lower.contains("ció") { s_ca += 2; }

    for tok in tokenize(&lower) {
        if EN_DETECT.binary_search(&tok.as_str()).is_ok() { s_en += 2; }
        if ES_DETECT.binary_search(&tok.as_str()).is_ok() { s_es += 2; }
        if CA_DETECT.binary_search(&tok.as_str()).is_ok() { s_ca += 2; }
        if EU_DETECT.binary_search(&tok.as_str()).is_ok() { s_eu += 2; }
    }

    let mut best = (s_en, Language::English);
    if s_es > best.0 { best = (s_es, Language::Spanish); }
    if s_ca > best.0 { best = (s_ca, Language::Catalan); }
    if s_eu > best.0 { best = (s_eu, Language::Basque); }
    best.1
}

const EN_DETECT: &[&str] = &[
    "a","able","about","above","actually","after","again","all","already","also","although","always",
    "am","an","and","another","any","are","as","at","back","bad","be","because","been","before",
    "being","below","between","big","both","but","by","came","can","cannot","could","did","didn",
    "different","do","does","doing","don","done","during","each","early","either","else","even",
    "every","few","first","following","for","from","get","go","going","gone","good","got","great",
    "had","has","have","he","her","here","him","his","how","however","i","if","im","important",
    "in","into","is","isn","it","its","itself","just","know","large","last","left","like","little",
    "long","look","looked","make","many","may","maybe","me","might","more","most","much","must",
    "my","myself","never","new","next","no","not","now","of","off","often","old","on","once",
    "one","only","or","other","ought","our","out","over","own","perhaps","please","probably",
    "public","really","right","said","same","say","says","see","seen","she","should","shall",
    "small","so","some","something","sometimes","still","such","sure","take","than","that","the",
    "their","them","then","there","these","they","this","those","though","through","thus","to",
    "today","together","too","took","twice","under","until","up","upon","us","use","used","using",
    "usually","very","want","was","wasn","we","well","went","were","what","whatever","when",
    "where","whether","which","while","who","whom","whose","why","will","with","without","won",
    "would","yes","yet","you","your","yours","yourself",
];
const ES_DETECT: &[&str] = &[
    "a", "acá", "ahora", "ahí", "al", "allá", "allí", "ante", "antes", "aquel", "aquella",
    "aquellas", "aquellos", "aquí", "así", "ayer", "aún", "bajo", "bien", "cada", "casi",
    "como", "con", "contra", "cual", "cuales", "cuando", "cuanta", "cuanto", "cuya", "cuyas",
    "cuyo", "cuyos", "cuál", "cuáles", "cuándo", "cuánta", "cuánto", "cómo", "de", "del",
    "desde", "donde", "durante", "dónde", "el", "ella", "ellas", "ello", "ellos", "en",
    "entre", "era", "eran", "eres", "es", "esa", "esas", "ese", "eso", "esos", "esta",
    "estamos", "estas", "este", "esto", "estos", "estoy", "está", "estáis", "están",
    "excepto", "fue", "fueron", "gracias", "ha", "había", "hace", "hacemos", "hacen",
    "hacia", "hacéis", "hago", "han", "hasta", "hay", "hola", "hoy", "jamás", "junto", "la",
    "las", "le", "les", "lo", "los", "luego", "mal", "mañana", "me", "mediante", "mejor",
    "mi", "mis", "mismo", "mucha", "muchas", "mucho", "muchos", "muy", "más", "nada",
    "nadie", "ni", "ninguna", "ninguno", "ningún", "no", "nos", "nosotros", "nuestra",
    "nuestras", "nuestro", "nuestros", "nunca", "o", "os", "otra", "otras", "otro", "otros",
    "para", "peor", "pero", "poca", "pocas", "poco", "pocos", "por", "porque", "porqué",
    "pronto", "pues", "que", "quien", "quién", "qué", "salvo", "se", "según", "señor",
    "señora", "si", "siempre", "sin", "sino", "sobre", "son", "soy", "su", "sus", "sí",
    "tal", "también", "tampoco", "tan", "tanto", "tarde", "te", "temprano", "tenemos",
    "tengo", "tenéis", "tiene", "tienen", "toda", "todas", "todavía", "todo", "todos",
    "tras", "tu", "tus", "tuyo", "ud", "uds", "un", "una", "unas", "uno", "unos", "usted",
    "ustedes", "vez", "y", "ya", "yo",
];
const CA_DETECT: &[&str] = &[
    "a", "acabar", "ací", "ahir", "així", "al", "algun", "alguna", "algunes", "alguns",
    "allà", "allí", "altra", "altre", "altres", "amb", "ambdós", "ans", "aquell", "aquella",
    "aquelles", "aquells", "aquest", "aquesta", "aquestes", "aquests", "ara", "avui", "açò",
    "bo", "bé", "cada", "cap", "cas", "cinc", "com", "contra", "còm", "d", "dalt", "damunt",
    "darrer", "de", "del", "dels", "dempeus", "des", "després", "deu", "dia", "dins",
    "dintre", "disset", "divuit", "doncs", "dos", "dues", "durant", "e", "el", "ell",
    "ella", "elles", "ells", "els", "em", "en", "encara", "entre", "era", "eren", "es",
    "esa", "esos", "esta", "estan", "estar", "estat", "està", "et", "ets", "fa", "fer",
    "feu", "fou", "gens", "gràcies", "ha", "haver", "havien", "hem", "hi", "ho", "hola",
    "ja", "jo", "l", "la", "les", "li", "llavors", "lo", "los", "ls", "m", "mai", "massa",
    "meu", "meus", "meva", "meves", "mi", "molt", "molta", "moltes", "molts", "més", "n",
    "nho", "ni", "ningú", "no", "nogensmenys", "nosaltres", "nostra", "nostre", "nostres",
    "o", "on", "onze", "pas", "pel", "pels", "per", "perquè", "però", "poc", "pocs",
    "poder", "podeu", "prou", "puc", "q", "qual", "quan", "que", "qui", "quin", "quina",
    "quines", "quins", "quinze", "què", "s", "sa", "sant", "se", "segons", "sempre",
    "sense", "ser", "ses", "seu", "seus", "seva", "seves", "si", "sobre", "soc", "som",
    "sota", "sou", "sí", "sóc", "són", "ta", "tal", "també", "tant", "tanta", "tantes",
    "tants", "te", "tenia", "teniu", "tens", "teu", "teus", "teva", "teves", "ti", "ton",
    "tot", "tota", "totes", "tots", "tres", "tu", "u", "un", "una", "unes", "uns", "us",
    "va", "vaig", "vam", "van", "vau", "veure", "vols", "volt", "vosaltres", "vostra",
    "vostre", "vostres", "y", "ya", "ès",
];
const EU_DETECT: &[&str] = &[
    "ala","alde","arte","atzean","aurka","aurre","aurrean","azpian","bada","baietz","baina",
    "bai","barruan","bat","beraz","berori","bestela","beti","bezala","bi","bitartean","da",
    "dago","daude","ditu","donostia","du","dute","edo","egin","egun","ere","etxe","etxea",
    "euskal","euskara","ez","ezetz","eta","gainean","gainera","gero","gisa","gu","gure","gutxi",
    "guzti","haien","haiek","haren","hau","hemen","han","herri","honela","hor","hori","hortxe",
    "inoiz","izan","izena","kale","kalean","kaleak","kanpoan","kontra","lehen","ni","nire",
    "noiz","nola","nondik","nor","nori","nork","ondo","ondoren","oraingo","orain","orduan",
    "oso","ostean","ta","tartean","tu","ti","te","tun","urte","zai","zenbat","zer",
    "zergatik","zion","zu","zuek","zuen","zure",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_english() {
        assert_eq!(detect_language("the quick brown fox jumps over the lazy dog"), Language::English);
    }

    #[test]
    fn detects_spanish() {
        assert_eq!(detect_language("¿Dónde está la biblioteca? El niño está aquí."), Language::Spanish);
    }

    #[test]
    fn detects_catalan() {
        assert_eq!(
            detect_language("Bon dia! Com estàs avui? Aquesta és la meva casa, una mica més enllà."),
            Language::Catalan
        );
    }

    #[test]
    fn detects_catalan_via_geminated_l() {
        assert_eq!(detect_language("paral·lel i col·lecció són dues paraules"), Language::Catalan);
    }

    #[test]
    fn detects_basque() {
        assert_eq!(detect_language("Etxean nago, gaur eskolara joango naiz."), Language::Basque);
    }

    #[test]
    fn lists_are_sorted() {
        for w in [EN_DETECT.windows(2), ES_DETECT.windows(2), CA_DETECT.windows(2), EU_DETECT.windows(2)] {
            for pair in w {
                assert!(pair[0] < pair[1], "lang detect list out of order near {:?}", pair);
            }
        }
    }
}
