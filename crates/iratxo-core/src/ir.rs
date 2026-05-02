use serde::{Deserialize, Serialize};

/// Compiled rule program. The unit a Wasm plugin evaluates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub description: String,
    pub rules: Vec<Rule>,
    pub default: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub when: Predicate,
    pub verdict: Verdict,
    /// IDs of rules to evaluate only if this rule triggers.
    /// Empty in the common case.
    #[serde(default)]
    pub then: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub classify: String,
    pub confidence: f32,
    pub explanation: Option<String>,
}

/// Pure, deterministic predicates over the input text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    /// At least one of the needles appears.
    ContainsAny { needles: Vec<String>, case_sensitive: bool },
    /// All needles appear.
    ContainsAll { needles: Vec<String>, case_sensitive: bool },
    /// None of the needles appear.
    NotContainsAny { needles: Vec<String>, case_sensitive: bool },
    /// Regex match against the input.
    Regex { pattern: String, case_sensitive: bool },
    /// Token count >= n.
    MinLength { tokens: u32 },
    /// Token count <= n.
    MaxLength { tokens: u32 },
    /// True if input is semantically similar to any example, above `threshold`.
    SemanticMatch {
        examples: Vec<String>,
        threshold: f32,
        extra_synonyms: Vec<(String, Vec<String>)>,
        language: Option<String>,
    },
    /// Markdown/HTML heading whose text matches one of the given strings
    /// (case-insensitive, trimmed). Recognizes `#`-style and `<h1>..<h6>`.
    HasSection { titles: Vec<String> },
    /// At least one entity of `kind` is present.
    HasEntity { kind: EntityKind, min_count: u32 },
    /// Paragraph count is in `[min, max]`. Either bound can be omitted.
    ParagraphCount { min: Option<u32>, max: Option<u32> },
    /// Average words-per-sentence is at most `max`. Useful for readability.
    MaxWordsPerSentence { max: u32 },
    /// Detected language matches one of these codes ("en", "es", "ca", "eu").
    LanguageIs { codes: Vec<String> },
    /// At least one URL in the input has a host matching one of `domains`.
    /// When `allow_subdomains` is true (default), `*.example.com` also matches
    /// `example.com`. Compose with `Not` for "URL to *unallowed* domain".
    HasUrlToDomain { domains: Vec<String>, allow_subdomains: bool },
    /// Fraction of alphabetic chars that are uppercase ≥ `min_ratio` (0..=1).
    /// Inputs with no letters never trigger.
    MostlyUppercase { min_ratio: f32 },
    /// Some whitespace-token of length ≥ `min_token_len` has Shannon entropy
    /// (over the empirical char distribution of that token) ≥ `min_bits`.
    /// Useful for catching API keys / random secrets not covered by a known
    /// vendor regex.
    TokenEntropyAbove { min_bits: f32, min_token_len: u32 },

    // ---------- v3 additions ----------

    /// At least one of `needles` appears as a whole word (Unicode word boundary).
    /// Stricter than `ContainsAny` — "cat" matches "the cat sat" but not "category".
    WordContainsAny { needles: Vec<String>, case_sensitive: bool },
    /// Trimmed input begins with one of the prefixes.
    StartsWithAny { prefixes: Vec<String>, case_sensitive: bool },
    /// Trimmed input ends with one of the suffixes.
    EndsWithAny { suffixes: Vec<String>, case_sensitive: bool },
    /// Number of sentences (split on `.` `!` `?`) is in `[min, max]`.
    SentenceCount { min: Option<u32>, max: Option<u32> },
    /// Character count is in `[min, max]`. Counts Unicode scalars.
    CharCount { min: Option<u32>, max: Option<u32> },
    /// Number of `\n`-separated lines is in `[min, max]`.
    LineCount { min: Option<u32>, max: Option<u32> },
    /// Fraction of non-whitespace chars that are digits ≥ `min_ratio`.
    DigitRatioAbove { min_ratio: f32 },
    /// Fraction of non-whitespace chars that are ASCII punctuation ≥ `min_ratio`.
    PunctuationRatioAbove { min_ratio: f32 },
    /// Some run of `min_run` consecutive identical chars exists. Catches
    /// "soooooo" (letter run), "!!!!!" (punct run), or "............".
    RepeatedCharRun { min_run: u32 },
    /// Some non-stopword token (after lowercasing) appears `min_count`+ times.
    /// Detects spammy repetition.
    RepeatedToken { min_count: u32 },
    /// Type-token ratio (unique-tokens / total-tokens) is *at most* `max_ratio`.
    /// Low TTR ⇒ low lexical diversity (e.g. spam, copy-paste).
    TypeTokenRatioBelow { max_ratio: f32 },
    /// Input contains at least one invisible / zero-width / BOM character.
    /// Used to flag phishing or homoglyph-style obfuscation.
    HasInvisibleChars,
    /// Input contains at least one token mixing chars from multiple Unicode
    /// scripts (e.g. Cyrillic 'а' inside Latin "PayPal"). Homoglyph defense.
    HasMixedScriptToken,
    /// Input contains characters from at least one of the listed scripts.
    /// Codes: `latin`, `cyrillic`, `greek`, `han`, `hiragana`, `katakana`,
    /// `hangul`, `arabic`, `hebrew`, `devanagari`, `thai`.
    ScriptIs { scripts: Vec<String> },

    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    Not(Box<Predicate>),
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Email,
    Phone,
    Url,
    Currency,
    // v3 additions
    IpAddress,
    CreditCard,
    Iban,
    DateIso,
    Hashtag,
    Mention,
    Emoji,
}
