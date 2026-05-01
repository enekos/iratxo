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
    /// Detected language matches one of these codes ("en", "es", "eu").
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
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    Not(Box<Predicate>),
    Always,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityKind {
    Email,
    Phone,
    Url,
    Currency,
}
