//! Heuristic language detection for English / Spanish / Catalan / Basque /
//! French / Italian / German / Dutch.
//!
//! Character-level signals (e.g. `ñ`, `ç`, accented vowels, `tx`/`tz` digraphs,
//! Catalan `l·l`, French `œ`/trailing `-ment`, German `ß`/umlauts, Italian
//! `gn`/`zione`, Dutch `ij`) plus per-language closed word lists.
//! Tie-breaks favor English.

use crate::text::tokenize::tokenize_iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    English,
    Spanish,
    Catalan,
    Basque,
    French,
    Italian,
    German,
    Dutch,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::Catalan => "ca",
            Language::Basque  => "eu",
            Language::French  => "fr",
            Language::Italian => "it",
            Language::German  => "de",
            Language::Dutch   => "nl",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" | "english" => Some(Language::English),
            "es" | "spanish" => Some(Language::Spanish),
            "ca" | "cat" | "catalan" | "valencian" | "val" => Some(Language::Catalan),
            "eu" | "basque"  => Some(Language::Basque),
            "fr" | "french"  => Some(Language::French),
            "it" | "italian" => Some(Language::Italian),
            "de" | "german" | "deutsch" => Some(Language::German),
            "nl" | "dutch" | "flemish" | "nld" => Some(Language::Dutch),
            _ => None,
        }
    }

    /// Iterate every supported language. Stable order; useful in tests and tooling.
    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::Spanish,
            Language::Catalan,
            Language::Basque,
            Language::French,
            Language::Italian,
            Language::German,
            Language::Dutch,
        ]
    }
}

pub fn detect_language(text: &str) -> Language {
    detect_language_lowered(&text.to_lowercase())
}

/// Same as [`detect_language`] but assumes `lower` is already lowercased.
/// Avoids the `to_lowercase()` allocation when the caller already has it.
pub fn detect_language_lowered(lower: &str) -> Language {
    let mut s_en = 0i32;
    let mut s_es = 0i32;
    let mut s_ca = 0i32;
    let mut s_eu = 0i32;
    let mut s_fr = 0i32;
    let mut s_it = 0i32;
    let mut s_de = 0i32;
    let mut s_nl = 0i32;

    for c in lower.chars() {
        match c {
            'ñ' | '¿' | '¡' => s_es += 10,
            'ß' => s_de += 12,
            'œ' | 'æ' => s_fr += 8,
            // `ç` is heavily used in both Catalan and French; balance accordingly.
            'ç' => { s_ca += 6; s_fr += 6; }
            'ä' | 'ö' => s_de += 6,
            'ü' => { s_de += 5; s_ca += 1; s_es += 1; }
            'à' => { s_ca += 4; s_fr += 4; s_it += 4; }
            'è' => { s_ca += 4; s_fr += 3; s_it += 4; }
            'ò' => { s_ca += 5; s_it += 4; }
            'ù' => { s_fr += 4; s_it += 3; }
            'â' | 'ê' | 'î' | 'ô' | 'û' => s_fr += 5,
            'ë' | 'ÿ' => s_fr += 3,
            // `é` is dense in French past participles ("résilié", "été") AND
            // Spanish stressed forms ("también", "está"). Award both.
            'é' => { s_es += 3; s_fr += 3; s_ca += 1; s_it += 1; }
            'á' | 'í' | 'ó' | 'ú' => { s_es += 3; s_ca += 1; s_it += 1; }
            'ï' => { s_ca += 3; s_fr += 2; }
            _ => {}
        }
    }
    // Strong unique multi-char signals (rare false-positive risk).
    if lower.contains("l·l") || lower.contains("l.l") { s_ca += 12; }
    if lower.contains("zione") || lower.contains("zioni") { s_it += 5; }
    if lower.contains("oeu") { s_fr += 4; }    // "coeur", "oeuf", "soeur"
    if lower.contains("ij") { s_nl += 4; }     // "mijn", "zijn", "wij"
    if lower.contains("ció") { s_ca += 3; }
    if lower.contains("ny") { s_ca += 2; }
    if lower.contains("tx") { s_eu += 5; s_ca += 1; }
    if lower.contains("tz") { s_eu += 3; }
    // Catalan postverbal clitics use hyphens ("envia-nos", "veure-hi"); Spanish
    // writes clitics fused (e.g. "decírnoslo"), so a hyphenated short clitic
    // is a high-precision Catalan signal.
    if lower.contains("-nos") || lower.contains("-vos") || lower.contains("-hi")
        || lower.contains("-ho") || lower.contains("-li") || lower.contains("-se ")
        || lower.contains("-ne ") { s_ca += 3; }
    // Medium signals: distinctive but possible cross-language hits.
    if lower.contains(" gli ") || lower.starts_with("gli ") { s_it += 3; }  // article "gli"
    if lower.contains("sch") { s_de += 2; s_nl += 2; }
    // Dropped: `tion`/`ment` (very common in English too — "comment", "moment", "government").
    // Dropped: `aa`/`ee`/`oo` (English collides — "look", "see", "good").
    // Dropped: `gn` (common in French — "ligne", "agneau").

    // Use tokenize_iter to avoid allocating a String per token.
    for tok in tokenize_iter(lower) {
        if EN_DETECT.binary_search(&tok).is_ok() { s_en += 2; }
        if ES_DETECT.binary_search(&tok).is_ok() { s_es += 2; }
        if CA_DETECT.binary_search(&tok).is_ok() { s_ca += 2; }
        if EU_DETECT.binary_search(&tok).is_ok() { s_eu += 2; }
        if FR_DETECT.binary_search(&tok).is_ok() { s_fr += 2; }
        if IT_DETECT.binary_search(&tok).is_ok() { s_it += 2; }
        if DE_DETECT.binary_search(&tok).is_ok() { s_de += 2; }
        if NL_DETECT.binary_search(&tok).is_ok() { s_nl += 2; }
    }

    let mut best = (s_en, Language::English);
    if s_es > best.0 { best = (s_es, Language::Spanish); }
    if s_ca > best.0 { best = (s_ca, Language::Catalan); }
    if s_eu > best.0 { best = (s_eu, Language::Basque); }
    if s_fr > best.0 { best = (s_fr, Language::French); }
    if s_it > best.0 { best = (s_it, Language::Italian); }
    if s_de > best.0 { best = (s_de, Language::German); }
    if s_nl > best.0 { best = (s_nl, Language::Dutch); }
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
    "public","really","right","said","same","say","says","see","seen","shall","she","should",
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
    "i", "ja", "jo", "l", "la", "les", "li", "llavors", "lo", "los", "ls", "m", "mai", "massa",
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
    "ala", "alde", "arte", "atzean", "aurka", "aurre", "aurrean", "azpian", "bada", "bai",
    "baietz", "baina", "barruan", "bat", "beraz", "berori", "bestela", "beti", "bezala", "bi",
    "bitartean", "da", "dago", "daude", "ditu", "donostia", "du", "dute", "edo", "egin", "egun",
    "ere", "eta", "etxe", "etxea", "euskal", "euskara", "ez", "ezetz", "gainean", "gainera",
    "gero", "gisa", "gu", "gure", "gutxi", "guzti", "haiek", "haien", "han", "haren", "hau",
    "hemen", "herri", "honela", "hor", "hori", "hortxe", "inoiz", "izan", "izena", "kale",
    "kaleak", "kalean", "kanpoan", "kontra", "lehen", "ni", "nire", "noiz", "nola", "nondik",
    "nor", "nori", "nork", "ondo", "ondoren", "orain", "oraingo", "orduan", "oso", "ostean",
    "ta", "tartean", "te", "ti", "tu", "tun", "urte", "zai", "zenbat", "zer", "zergatik",
    "zion", "zu", "zuek", "zuen", "zure",
];
const FR_DETECT: &[&str] = &[
    "a", "alors", "après", "as", "au", "aucun", "aussi", "autre", "aux", "avant", "avec", "avoir",
    "bien", "bon", "bonjour", "car", "ce", "ceci", "cela", "celle", "celles", "celui", "ces",
    "cet", "cette", "ceux", "chaque", "comme", "comment", "d", "dans", "de", "des", "donc",
    "dont", "du", "elle", "elles", "en", "encore", "entre", "es", "est", "et", "eu",
    "faire", "fait", "fois", "hier", "ici", "il", "ils", "j", "je", "juste", "l",
    "la", "le", "les", "leur", "leurs", "lui", "là", "m", "ma", "maintenant", "mais",
    "me", "merci", "mes", "moi", "mon", "même", "n", "ne", "ni", "non", "notre",
    "nous", "on", "ont", "ou", "oui", "où", "par", "parce", "pas", "peu", "peut",
    "plus", "pour", "pourquoi", "qu", "quand", "que", "quel", "quelle", "quelles", "quels", "qui",
    "quoi", "s", "sa", "sans", "se", "ses", "seulement", "si", "sien", "soi", "soit",
    "son", "sont", "sous", "sur", "ta", "te", "tes", "toi", "ton", "tous", "tout",
    "toute", "toutes", "très", "tu", "un", "une", "voici", "voilà", "vos", "votre", "vous",
    "vraiment", "y", "ça", "étaient", "était", "été", "être",
];
const IT_DETECT: &[&str] = &[
    "a", "abbiamo", "ad", "adesso", "agli", "ai", "al", "alcuni", "alla", "alle", "allo",
    "altre", "altri", "altro", "anche", "ancora", "avere", "aveva", "bene", "buono", "c", "che",
    "chi", "ci", "ciao", "cioè", "co", "come", "con", "contro", "cosa", "così", "cui",
    "d", "da", "dagli", "dai", "dal", "dalla", "dalle", "dallo", "davvero", "degli", "dei",
    "del", "della", "delle", "dello", "dentro", "di", "dietro", "domani", "dopo", "dove", "dovuto",
    "due", "dunque", "e", "ecco", "ed", "egli", "ella", "era", "erano", "essere", "fa",
    "fare", "fatto", "fra", "gli", "grazie", "ha", "hai", "hanno", "ho", "i", "il",
    "in", "io", "l", "la", "le", "lei", "li", "lo", "loro", "lui", "ma",
    "mai", "me", "meglio", "mentre", "mi", "mia", "mie", "miei", "mio", "molto", "ne",
    "negli", "nei", "nel", "nella", "nelle", "nello", "no", "noi", "non", "nostra", "nostre",
    "nostri", "nostro", "o", "oggi", "ogni", "ora", "per", "perché", "però", "più", "poi",
    "prima", "qua", "qual", "quale", "quali", "quando", "quanto", "quei", "quel", "quella", "quelle",
    "quelli", "quello", "questa", "queste", "questi", "questo", "qui", "se", "sei", "senza", "siamo",
    "siete", "sono", "sopra", "sotto", "stato", "stesso", "su", "sua", "sue", "sugli", "sui",
    "sul", "sulla", "sulle", "sullo", "suo", "suoi", "te", "ti", "tra", "tu", "tua",
    "tue", "tuo", "tuoi", "tutta", "tutte", "tutti", "tutto", "un", "una", "uno", "voi",
    "vostra", "vostre", "vostri", "vostro",
];
const DE_DETECT: &[&str] = &[
    "aber", "alle", "allem", "allen", "aller", "alles", "als", "also", "am", "an", "ander",
    "anders", "auch", "auf", "aus", "bei", "bin", "bis", "bist", "da", "dadurch", "daher",
    "danach", "dann", "das", "dass", "dazu", "daß", "dein", "deine", "dem", "den", "der",
    "des", "deshalb", "dessen", "die", "dies", "dieser", "dieses", "doch", "dort", "du", "durch",
    "ein", "eine", "einem", "einen", "einer", "eines", "er", "es", "etwa", "etwas", "euch",
    "euer", "eure", "für", "gegen", "gewesen", "habe", "haben", "hat", "hatte", "hatten", "heute",
    "hier", "hinter", "ich", "ihm", "ihn", "ihnen", "ihr", "ihre", "im", "immer", "in",
    "indem", "ist", "ja", "jede", "jeden", "jedoch", "jener", "jenes", "jetzt", "kann", "kein",
    "keine", "können", "könnte", "machen", "man", "manche", "mein", "meine", "mit", "morgen", "muss",
    "mußt", "müssen", "müßt", "nach", "nachdem", "nein", "nicht", "nichts", "noch", "nun", "nur",
    "ob", "oder", "ohne", "schon", "sehr", "sein", "seine", "seinem", "seinen", "seiner", "seines",
    "sich", "sie", "sind", "so", "sollte", "sondern", "sonst", "soweit", "sowie", "um", "und",
    "uns", "unser", "unsere", "unter", "viel", "vom", "von", "vor", "wann", "war", "waren",
    "warum", "was", "weil", "weiter", "weiteren", "welche", "wem", "wen", "wenn", "wer", "werde",
    "werden", "wie", "wieder", "will", "wir", "wird", "wo", "während", "wäre", "würde", "würden",
    "zu", "zum", "zur", "zwar", "zwischen", "über",
];
const NL_DETECT: &[&str] = &[
    "aan", "af", "al", "alle", "alleen", "als", "altijd", "andere", "anders", "ben", "bij",
    "binnen", "boven", "buiten", "daar", "dan", "dat", "de", "deze", "die", "dit", "doe",
    "doen", "door", "dus", "een", "eens", "elk", "elke", "en", "enkele", "er", "even",
    "ga", "gaan", "geen", "gisteren", "goed", "graag", "haar", "had", "hadden", "heb", "hebben",
    "heeft", "hem", "hen", "het", "hier", "hij", "hoe", "hoeveel", "hun", "iemand", "iets",
    "ik", "in", "is", "ja", "je", "jij", "jou", "jouw", "jullie", "kan", "kon",
    "kunnen", "maar", "mag", "me", "meer", "meest", "men", "met", "mij", "mijn", "moet",
    "moeten", "mooi", "morgen", "na", "naar", "nee", "niemand", "niet", "niets", "nog", "nu",
    "of", "om", "omdat", "ons", "onze", "ook", "op", "over", "te", "tegen", "toen",
    "tot", "tussen", "u", "uit", "uw", "vaak", "van", "vandaag", "veel", "vlak", "voor",
    "waar", "waarom", "wanneer", "want", "waren", "was", "wat", "we", "wel", "welk", "welke",
    "werd", "werden", "wie", "wij", "wilde", "willen", "worden", "wordt", "ze", "zelf", "zich",
    "zij", "zijn", "zo", "zonder", "zou", "zouden",
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
    fn detects_french() {
        assert_eq!(
            detect_language("Bonjour, je m'appelle Pierre et j'habite à Paris avec mes amis."),
            Language::French
        );
    }

    #[test]
    fn detects_italian() {
        assert_eq!(
            detect_language("Buongiorno, mi chiamo Marco e abito a Roma con la mia famiglia."),
            Language::Italian
        );
    }

    #[test]
    fn detects_german() {
        assert_eq!(
            detect_language("Guten Tag, ich heiße Hans und ich wohne in München mit meiner Familie."),
            Language::German
        );
    }

    #[test]
    fn detects_dutch() {
        assert_eq!(
            detect_language("Goedendag, ik heet Jan en ik woon in Amsterdam met mijn familie."),
            Language::Dutch
        );
    }

    #[test]
    fn lists_are_sorted() {
        for w in [
            EN_DETECT.windows(2), ES_DETECT.windows(2), CA_DETECT.windows(2), EU_DETECT.windows(2),
            FR_DETECT.windows(2), IT_DETECT.windows(2), DE_DETECT.windows(2), NL_DETECT.windows(2),
        ] {
            for pair in w {
                assert!(pair[0] < pair[1], "lang detect list out of order near {:?}", pair);
            }
        }
    }

    // Closely-related language disambiguation. The "shared-substrate" pairs are
    // where naive heuristics fail: Spanish↔Catalan, French↔Italian, German↔Dutch
    // share function words and accented letters. These tests pin down behavior
    // so refactors of the heuristics surface regressions.

    #[test]
    fn disambiguates_spanish_vs_catalan() {
        assert_eq!(detect_language("Quiero cancelar mi suscripción mañana."), Language::Spanish);
        assert_eq!(detect_language("Vull cancel·lar la meva subscripció demà."), Language::Catalan);
    }

    #[test]
    fn disambiguates_french_vs_italian() {
        assert_eq!(detect_language("Je voudrais résilier mon contrat le mois prochain."), Language::French);
        assert_eq!(detect_language("Vorrei cancellare il mio contratto il mese prossimo."), Language::Italian);
    }

    #[test]
    fn disambiguates_german_vs_dutch() {
        assert_eq!(detect_language("Ich möchte mein Abonnement morgen kündigen."), Language::German);
        assert_eq!(detect_language("Ik wil mijn abonnement morgen opzeggen."), Language::Dutch);
    }

    #[test]
    fn disambiguates_french_vs_english_on_short_text() {
        // Both contain "comment" — used to be a problem with the dropped
        // `tion`/`ment` heuristics tilting toward French.
        assert_eq!(detect_language("I have a comment about the moment of intervention."), Language::English);
        assert_eq!(detect_language("J'ai un commentaire sur le moment de l'intervention."), Language::French);
    }

    #[test]
    fn dutch_doesnt_lose_to_english_on_oo_ee() {
        // "school" and "see" used to wrongly give Dutch a digraph bonus that
        // could tip short English sentences. Verify English wins.
        assert_eq!(detect_language("I see the school is good today."), Language::English);
        // And clear Dutch still wins.
        assert_eq!(detect_language("Ik zie dat de school vandaag goed is."), Language::Dutch);
    }

    #[test]
    fn from_code_handles_all_aliases() {
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("english"), Some(Language::English));
        assert_eq!(Language::from_code("es"), Some(Language::Spanish));
        assert_eq!(Language::from_code("spanish"), Some(Language::Spanish));
        assert_eq!(Language::from_code("ca"), Some(Language::Catalan));
        assert_eq!(Language::from_code("cat"), Some(Language::Catalan));
        assert_eq!(Language::from_code("catalan"), Some(Language::Catalan));
        assert_eq!(Language::from_code("valencian"), Some(Language::Catalan));
        assert_eq!(Language::from_code("val"), Some(Language::Catalan));
        assert_eq!(Language::from_code("eu"), Some(Language::Basque));
        assert_eq!(Language::from_code("basque"), Some(Language::Basque));
        assert_eq!(Language::from_code("fr"), Some(Language::French));
        assert_eq!(Language::from_code("french"), Some(Language::French));
        assert_eq!(Language::from_code("it"), Some(Language::Italian));
        assert_eq!(Language::from_code("italian"), Some(Language::Italian));
        assert_eq!(Language::from_code("de"), Some(Language::German));
        assert_eq!(Language::from_code("german"), Some(Language::German));
        assert_eq!(Language::from_code("deutsch"), Some(Language::German));
        assert_eq!(Language::from_code("nl"), Some(Language::Dutch));
        assert_eq!(Language::from_code("nld"), Some(Language::Dutch));
        assert_eq!(Language::from_code("dutch"), Some(Language::Dutch));
        assert_eq!(Language::from_code("flemish"), Some(Language::Dutch));
        assert_eq!(Language::from_code("xx"), None);
        assert_eq!(Language::from_code(""), None);
    }

    #[test]
    fn code_and_from_code_roundtrip_for_all_variants() {
        for &lang in Language::all() {
            assert_eq!(Language::from_code(lang.code()), Some(lang));
        }
        assert_eq!(Language::all().len(), 8);
    }
}
