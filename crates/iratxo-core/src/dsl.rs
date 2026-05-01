use crate::ir::{EntityKind, Predicate, Program, Rule, Verdict};
use serde::Deserialize;
use std::fmt;

#[derive(Debug)]
pub enum DslError {
    /// Source-mapped YAML/parse error. `line` and `column` are 1-based when
    /// known. `snippet` is the offending source line if we could recover it.
    Parse {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
        snippet: Option<String>,
    },
    Validation(String),
}

impl fmt::Display for DslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DslError::Parse { message, line, column, snippet } => {
                match (line, column) {
                    (Some(l), Some(c)) => write!(f, "parse error at line {l}, column {c}: {message}")?,
                    (Some(l), None)    => write!(f, "parse error at line {l}: {message}")?,
                    _                  => write!(f, "parse error: {message}")?,
                }
                if let Some(s) = snippet {
                    write!(f, "\n  | {}", s)?;
                    if let Some(c) = column {
                        write!(f, "\n  | {}^", " ".repeat(c.saturating_sub(1)))?;
                    }
                }
                Ok(())
            }
            DslError::Validation(s) => write!(f, "validation error: {}", s),
        }
    }
}

impl std::error::Error for DslError {}

impl DslError {
    /// Convert a `serde_yaml::Error` to a source-mapped `DslError`, pulling
    /// the offending line out of `src` for context.
    pub fn from_yaml(e: serde_yaml::Error, src: &str) -> Self {
        let location = e.location();
        let (line, column) = match &location {
            Some(loc) => (Some(loc.line()), Some(loc.column())),
            None => (None, None),
        };
        let snippet = line.and_then(|l| src.lines().nth(l.saturating_sub(1)).map(str::to_string));
        DslError::Parse {
            message: e.to_string(),
            line,
            column,
            snippet,
        }
    }
}

#[derive(Deserialize)]
struct DslDoc {
    name: String,
    #[serde(default)]
    description: String,
    rules: Vec<DslRule>,
    #[serde(default)]
    default: Option<DslVerdict>,
}

#[derive(Deserialize)]
struct DslRule {
    id: String,
    when: DslPredicate,
    classify: String,
    #[serde(default = "default_confidence")]
    confidence: f32,
    #[serde(default)]
    explanation: Option<String>,
    /// IDs of rules to also evaluate when this rule triggers.
    #[serde(default)]
    then: Vec<String>,
}

fn default_confidence() -> f32 { 1.0 }

#[derive(Deserialize)]
struct DslVerdict {
    classify: String,
    #[serde(default = "default_confidence")]
    confidence: f32,
    #[serde(default)]
    explanation: Option<String>,
}

/// All predicate forms accepted in YAML. Each variant maps 1:1 to `ir::Predicate`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslPredicate {
    #[serde(default)]
    contains_any: Option<Vec<String>>,
    #[serde(default)]
    contains_all: Option<Vec<String>>,
    #[serde(default)]
    not_contains_any: Option<Vec<String>>,
    #[serde(default)]
    regex: Option<String>,
    #[serde(default)]
    min_length: Option<u32>,
    #[serde(default)]
    max_length: Option<u32>,
    #[serde(default)]
    all: Option<Vec<DslPredicate>>,
    #[serde(default)]
    any: Option<Vec<DslPredicate>>,
    #[serde(default)]
    not: Option<Box<DslPredicate>>,
    #[serde(default)]
    semantic_match: Option<DslSemantic>,
    #[serde(default)]
    has_section: Option<Vec<String>>,
    #[serde(default)]
    has_entity: Option<DslEntity>,
    #[serde(default)]
    paragraphs: Option<DslRange>,
    #[serde(default)]
    max_words_per_sentence: Option<u32>,
    #[serde(default)]
    language_is: Option<Vec<String>>,
    #[serde(default)]
    has_url_to_domain: Option<DslUrlDomain>,
    #[serde(default)]
    mostly_uppercase: Option<DslRatio>,
    #[serde(default)]
    token_entropy_above: Option<DslEntropy>,
    /// Tautology — useful in `then`-chained rules whose firing depends only
    /// on the parent rule, not on a separate input check.
    #[serde(default)]
    always: bool,
    #[serde(default = "default_case_sensitive")]
    case_sensitive: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslUrlDomain {
    domains: Vec<String>,
    #[serde(default = "default_allow_subdomains")]
    allow_subdomains: bool,
}

fn default_allow_subdomains() -> bool { true }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslRatio {
    min_ratio: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslEntropy {
    min_bits: f32,
    #[serde(default = "default_min_token_len")]
    min_token_len: u32,
}

fn default_min_token_len() -> u32 { 16 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslEntity {
    kind: String,
    #[serde(default = "default_min_count")]
    min_count: u32,
}

fn default_min_count() -> u32 { 1 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslRange {
    #[serde(default)]
    min: Option<u32>,
    #[serde(default)]
    max: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslSemantic {
    examples: Vec<String>,
    #[serde(default = "default_threshold")]
    threshold: f32,
    /// Optional user-supplied dictionary: canonical -> [synonyms].
    #[serde(default)]
    synonyms: std::collections::BTreeMap<String, Vec<String>>,
    /// Optional language hint: "en", "es", "eu", or omitted for auto-detect.
    #[serde(default)]
    language: Option<String>,
}

fn default_threshold() -> f32 { 0.4 }

fn default_case_sensitive() -> bool { false }

pub fn parse(src: &str) -> Result<Program, DslError> {
    let doc: DslDoc = serde_yaml::from_str(src).map_err(|e| DslError::from_yaml(e, src))?;
    if doc.rules.is_empty() {
        return Err(DslError::Validation("at least one rule required".into()));
    }
    let rules = doc.rules.into_iter().map(lower_rule).collect::<Result<Vec<_>, _>>()?;
    let default = doc.default.map(lower_verdict).unwrap_or(Verdict {
        classify: "ok".into(),
        confidence: 1.0,
        explanation: None,
    });
    Ok(Program {
        name: doc.name,
        description: doc.description,
        rules,
        default,
    })
}

fn lower_rule(r: DslRule) -> Result<Rule, DslError> {
    Ok(Rule {
        id: r.id,
        when: lower_predicate(r.when)?,
        verdict: Verdict { classify: r.classify, confidence: r.confidence, explanation: r.explanation },
        then: r.then,
    })
}

fn lower_verdict(v: DslVerdict) -> Verdict {
    Verdict { classify: v.classify, confidence: v.confidence, explanation: v.explanation }
}

fn lower_predicate(p: DslPredicate) -> Result<Predicate, DslError> {
    let cs = p.case_sensitive;
    let mut variants: Vec<Predicate> = Vec::new();
    if let Some(n) = p.contains_any { variants.push(Predicate::ContainsAny { needles: n, case_sensitive: cs }); }
    if let Some(n) = p.contains_all { variants.push(Predicate::ContainsAll { needles: n, case_sensitive: cs }); }
    if let Some(n) = p.not_contains_any { variants.push(Predicate::NotContainsAny { needles: n, case_sensitive: cs }); }
    if let Some(r) = p.regex { variants.push(Predicate::Regex { pattern: r, case_sensitive: cs }); }
    if let Some(n) = p.min_length { variants.push(Predicate::MinLength { tokens: n }); }
    if let Some(n) = p.max_length { variants.push(Predicate::MaxLength { tokens: n }); }
    if let Some(items) = p.all {
        let lowered = items.into_iter().map(lower_predicate).collect::<Result<Vec<_>, _>>()?;
        variants.push(Predicate::All(lowered));
    }
    if let Some(items) = p.any {
        let lowered = items.into_iter().map(lower_predicate).collect::<Result<Vec<_>, _>>()?;
        variants.push(Predicate::Any(lowered));
    }
    if let Some(inner) = p.not {
        variants.push(Predicate::Not(Box::new(lower_predicate(*inner)?)));
    }
    if let Some(titles) = p.has_section {
        if titles.is_empty() {
            return Err(DslError::Validation("has_section requires at least one title".into()));
        }
        variants.push(Predicate::HasSection { titles });
    }
    if let Some(e) = p.has_entity {
        let kind = match e.kind.as_str() {
            "email"    => EntityKind::Email,
            "phone"    => EntityKind::Phone,
            "url"      => EntityKind::Url,
            "currency" => EntityKind::Currency,
            other      => return Err(DslError::Validation(format!("unknown entity kind: {} (use email|phone|url|currency)", other))),
        };
        variants.push(Predicate::HasEntity { kind, min_count: e.min_count });
    }
    if let Some(r) = p.paragraphs {
        if r.min.is_none() && r.max.is_none() {
            return Err(DslError::Validation("paragraphs requires at least one of min/max".into()));
        }
        variants.push(Predicate::ParagraphCount { min: r.min, max: r.max });
    }
    if let Some(n) = p.max_words_per_sentence {
        variants.push(Predicate::MaxWordsPerSentence { max: n });
    }
    if let Some(codes) = p.language_is {
        if codes.is_empty() {
            return Err(DslError::Validation("language_is requires at least one code".into()));
        }
        for c in &codes {
            if crate::text::Language::from_code(c).is_none() {
                return Err(DslError::Validation(format!("unsupported language code: {} (use en|es|eu)", c)));
            }
        }
        variants.push(Predicate::LanguageIs { codes });
    }
    if let Some(u) = p.has_url_to_domain {
        if u.domains.is_empty() {
            return Err(DslError::Validation("has_url_to_domain requires at least one domain".into()));
        }
        let domains = u.domains.into_iter().map(|d| d.trim().trim_start_matches('.').to_lowercase()).collect();
        variants.push(Predicate::HasUrlToDomain { domains, allow_subdomains: u.allow_subdomains });
    }
    if let Some(r) = p.mostly_uppercase {
        if !(0.0..=1.0).contains(&r.min_ratio) {
            return Err(DslError::Validation(format!("mostly_uppercase min_ratio must be in [0,1], got {}", r.min_ratio)));
        }
        variants.push(Predicate::MostlyUppercase { min_ratio: r.min_ratio });
    }
    if let Some(e) = p.token_entropy_above {
        if e.min_bits < 0.0 {
            return Err(DslError::Validation(format!("token_entropy_above min_bits must be >= 0, got {}", e.min_bits)));
        }
        if e.min_token_len == 0 {
            return Err(DslError::Validation("token_entropy_above min_token_len must be > 0".into()));
        }
        variants.push(Predicate::TokenEntropyAbove { min_bits: e.min_bits, min_token_len: e.min_token_len });
    }
    if p.always {
        variants.push(Predicate::Always);
    }
    if let Some(s) = p.semantic_match {
        if s.examples.is_empty() {
            return Err(DslError::Validation("semantic_match requires at least one example".into()));
        }
        if !(0.0..=1.0).contains(&s.threshold) {
            return Err(DslError::Validation(format!("semantic_match threshold must be in [0,1], got {}", s.threshold)));
        }
        let extra = s.synonyms.into_iter().collect();
        if let Some(ref code) = s.language {
            if crate::text::Language::from_code(code).is_none() {
                return Err(DslError::Validation(format!("unsupported language: {} (use en|es|eu)", code)));
            }
        }
        variants.push(Predicate::SemanticMatch {
            examples: s.examples,
            threshold: s.threshold,
            extra_synonyms: extra,
            language: s.language,
        });
    }

    match variants.len() {
        0 => Err(DslError::Validation("predicate must specify at least one of: contains_any/contains_all/not_contains_any/regex/min_length/max_length/all/any/not".into())),
        1 => Ok(variants.into_iter().next().unwrap()),
        _ => Ok(Predicate::All(variants)),
    }
}
