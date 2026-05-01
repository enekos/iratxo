//! Heuristic language detection for English / Spanish / Basque.
//!
//! Ported from marrow's stemmer/detectwords.go. Character-level signals
//! (e.g. `ñ`, accented vowels, `tx`/`tz` digraphs) plus per-language closed
//! word lists. Tie-breaks favor English.

use crate::text::tokenize::tokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    English,
    Spanish,
    Basque,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::Basque => "eu",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" | "english" => Some(Language::English),
            "es" | "spanish" => Some(Language::Spanish),
            "eu" | "basque"  => Some(Language::Basque),
            _ => None,
        }
    }
}

pub fn detect_language(text: &str) -> Language {
    let lower = text.to_lowercase();
    let mut s_en = 0i32;
    let mut s_es = 0i32;
    let mut s_eu = 0i32;

    for c in lower.chars() {
        match c {
            'ñ' | '¿' | '¡' => s_es += 10,
            'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü' => s_es += 5,
            _ => {}
        }
    }
    if lower.contains("tx") { s_eu += 5; }
    if lower.contains("tz") { s_eu += 3; }

    for tok in tokenize(&lower) {
        if EN_DETECT.binary_search(&tok.as_str()).is_ok() { s_en += 2; }
        if ES_DETECT.binary_search(&tok.as_str()).is_ok() { s_es += 2; }
        if EU_DETECT.binary_search(&tok.as_str()).is_ok() { s_eu += 2; }
    }

    let mut best = (s_en, Language::English);
    if s_es > best.0 { best = (s_es, Language::Spanish); }
    if s_eu > best.0 { best = (s_eu, Language::Basque); }
    best.1
}

// Sorted slices so we can binary_search. Ported from marrow detectwords.go.
const EN_DETECT: &[&str] = &[
    "a","able","above","actually","after","again","all","already","also","always",
    "an","and","any","are","as","at","back","bad","be","because","been","before",
    "being","below","between","big","both","but","by","can","could","did","different",
    "do","does","during","each","early","even","few","first","following","for","from",
    "good","great","had","has","have","he","her","here","him","his","how","i","if",
    "important","in","into","is","it","its","just","large","last","left","little","long",
    "may","maybe","me","might","more","most","must","my","never","new","next","no","not",
    "now","of","often","old","on","once","only","or","other","ought","our","own","perhaps",
    "probably","public","really","right","same","she","should","shall","small","so","some",
    "sometimes","still","such","sure","than","that","the","their","them","then","there",
    "these","they","this","those","though","through","to","too","twice","us","usually","very",
    "was","we","well","were","what","whatever","when","where","which","while","who","whom",
    "whose","why","will","with","would","yes","yet","you","young","your",
];
const ES_DETECT: &[&str] = &[
    "a","acá","ahí","ahora","al","allá","allí","ante","antes","antes","aquel","aquella",
    "aquellas","aquellos","aquí","aún","bajo","bien","como","con","cual","cuál","cuales",
    "cuáles","cuando","cuándo","cuanta","cuánta","cuanto","cuánto","cuya","cuyas","cuyo",
    "cuyos","cómo","de","del","desde","donde","dónde","durante","el","en","entre","es",
    "esa","esas","ese","eso","esos","esta","están","estamos","estas","estáis","este","esto",
    "estos","estoy","está","excepto","fue","fueron","ha","había","hace","hacemos","hacen",
    "hacéis","hago","han","hasta","hay","hacia","jamás","la","las","le","les","lo","los",
    "luego","mal","más","me","mediante","mejor","mi","mis","mucha","muchas","mucho","muchos",
    "muy","no","nos","nuestra","nuestras","nuestro","nuestros","nunca","o","os","para","pero",
    "peor","poca","pocas","poco","pocos","por","porqué","porque","pronto","pues","qué","que",
    "quien","quién","salvo","se","según","si","siempre","sin","sino","sobre","son","su","sus",
    "sí","tarde","te","temprano","tenéis","tenemos","tengo","tiene","tienen","toda","todas",
    "todavía","todo","todos","tras","tu","tus","también","un","una","unas","unos","y","ya",
];
const EU_DETECT: &[&str] = &[
    "ala","alde","arte","atzean","aurka","aurre","aurrean","azpian","bada","baietz","baina",
    "bai","barruan","bat","beraz","berori","bestela","beti","bezala","bi","bitartean","da",
    "dago","daude","ditu","donostia","du","dute","edo","egin","egun","ere","etxe","etxea",
    "euskal","euskara","ez","ezetz","eta","gainean","gainera","gero","gisa","gu","gure","gutxi",
    "guzti","haien","haiek","haren","hau","hemen","han","herri","honela","hor","hori","hortxe",
    "inoiz","izan","izena","kale","kalean","kaleak","kanpoan","kontra","lehen","ni","nire",
    "noiz","nola","nondik","nor","nori","nork","ondo","ondoren","oraingo","orain","orduan",
    "oso","ostean","ta","tartean","tu","ti","te","te","tun","urte","zai","zenbat","zer",
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
    fn detects_basque() {
        assert_eq!(detect_language("Etxean nago, gaur eskolara joango naiz."), Language::Basque);
    }
}
