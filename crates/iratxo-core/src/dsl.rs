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
    // ---- v3 surface ----
    #[serde(default)]
    word_contains_any: Option<Vec<String>>,
    #[serde(default)]
    starts_with_any: Option<Vec<String>>,
    #[serde(default)]
    ends_with_any: Option<Vec<String>>,
    #[serde(default)]
    sentences: Option<DslRange>,
    #[serde(default)]
    chars: Option<DslRange>,
    #[serde(default)]
    lines: Option<DslRange>,
    #[serde(default)]
    digit_ratio_above: Option<DslRatio>,
    #[serde(default)]
    punctuation_ratio_above: Option<DslRatio>,
    #[serde(default)]
    repeated_char_run: Option<DslMinRun>,
    #[serde(default)]
    repeated_token: Option<DslMinCount>,
    #[serde(default)]
    type_token_ratio_below: Option<DslMaxRatio>,
    #[serde(default)]
    has_invisible_chars: bool,
    #[serde(default)]
    has_mixed_script_token: bool,
    #[serde(default)]
    script_is: Option<Vec<String>>,
    /// Tautology — useful in `then`-chained rules whose firing depends only
    /// on the parent rule, not on a separate input check.
    #[serde(default)]
    always: bool,
    #[serde(default = "default_case_sensitive")]
    case_sensitive: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslMinRun { min_run: u32 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslMinCount { min_count: u32 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DslMaxRatio { max_ratio: f32 }

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

fn is_known_script(s: &str) -> bool {
    matches!(
        s,
        "latin" | "cyrillic" | "greek" | "han" | "hiragana" | "katakana" |
        "hangul" | "arabic" | "hebrew" | "devanagari" | "thai"
    )
}

pub fn parse(src: &str) -> Result<Program, DslError> {
    let doc: DslDoc = serde_yaml::from_str(src).map_err(|e| DslError::from_yaml(e, src))?;
    if doc.name.trim().is_empty() {
        return Err(DslError::Validation("pack name is empty".into()));
    }
    if doc.rules.is_empty() {
        return Err(DslError::Validation("at least one rule required".into()));
    }
    let rules = doc.rules.into_iter().map(lower_rule).collect::<Result<Vec<_>, _>>()?;
    let default = match doc.default.map(lower_verdict) {
        Some(v) => {
            if v.classify.is_empty() {
                return Err(DslError::Validation("default classify is empty".into()));
            }
            if !(0.0..=1.0).contains(&v.confidence) {
                return Err(DslError::Validation(format!(
                    "default confidence {} out of [0,1]", v.confidence
                )));
            }
            v
        }
        None => Verdict { classify: "ok".into(), confidence: 1.0, explanation: None },
    };

    // Duplicate rule ids are a content-lint error: rules are looked up by id
    // in `then`-chains and tests, so collisions silently shadow each other.
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for r in &rules {
        if !seen.insert(r.id.as_str()) {
            return Err(DslError::Validation(format!(
                "duplicate rule id: {:?}", r.id
            )));
        }
    }
    // `then`-chained rule ids must reference real rules. Self-chains loop and
    // are caught here too.
    for r in &rules {
        for t in &r.then {
            if t == &r.id {
                return Err(DslError::Validation(format!(
                    "rule {:?} chains to itself in `then`", r.id
                )));
            }
            if !seen.contains(t.as_str()) {
                return Err(DslError::Validation(format!(
                    "rule {:?} chains to unknown rule {:?} in `then`", r.id, t
                )));
            }
        }
    }

    let chained_targets: Vec<String> = rules
        .iter()
        .flat_map(|r| r.then.iter().cloned())
        .collect();

    Ok(Program {
        name: doc.name,
        description: doc.description,
        rules,
        default,
        chained_targets,
    })
}

fn lower_rule(r: DslRule) -> Result<Rule, DslError> {
    if r.id.trim().is_empty() {
        return Err(DslError::Validation("rule id is empty".into()));
    }
    if r.classify.trim().is_empty() {
        return Err(DslError::Validation(format!("rule {:?}: classify is empty", r.id)));
    }
    if !(0.0..=1.0).contains(&r.confidence) {
        return Err(DslError::Validation(format!(
            "rule {:?}: confidence {} out of [0,1]", r.id, r.confidence
        )));
    }
    let id = r.id;
    let when = lower_predicate(r.when).map_err(|e| match e {
        DslError::Validation(msg) => DslError::Validation(format!("rule {:?}: {}", id, msg)),
        other => other,
    })?;
    Ok(Rule {
        id,
        when,
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
    if let Some(n) = p.contains_any {
        let needles = if cs { n } else { n.into_iter().map(|s| s.to_lowercase()).collect() };
        variants.push(Predicate::ContainsAny { needles, case_sensitive: cs });
    }
    if let Some(n) = p.contains_all {
        let needles = if cs { n } else { n.into_iter().map(|s| s.to_lowercase()).collect() };
        variants.push(Predicate::ContainsAll { needles, case_sensitive: cs });
    }
    if let Some(n) = p.not_contains_any {
        let needles = if cs { n } else { n.into_iter().map(|s| s.to_lowercase()).collect() };
        variants.push(Predicate::NotContainsAny { needles, case_sensitive: cs });
    }
    if let Some(r) = p.regex {
        let mut b = regex::RegexBuilder::new(&r);
        b.case_insensitive(!cs);
        if let Err(e) = b.build() {
            return Err(DslError::Validation(format!(
                "regex pattern {:?} failed to compile: {}", r, e
            )));
        }
        variants.push(Predicate::Regex { pattern: r, case_sensitive: cs });
    }
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
        let titles = titles.into_iter().map(|t| t.trim().to_lowercase()).collect();
        variants.push(Predicate::HasSection { titles });
    }
    if let Some(e) = p.has_entity {
        let kind = match e.kind.as_str() {
            "email"        => EntityKind::Email,
            "phone"        => EntityKind::Phone,
            "url"          => EntityKind::Url,
            "currency"     => EntityKind::Currency,
            "ip"           | "ip_address"  => EntityKind::IpAddress,
            "credit_card"  | "creditcard"  => EntityKind::CreditCard,
            "iban"                         => EntityKind::Iban,
            "date"         | "date_iso"    => EntityKind::DateIso,
            "hashtag"                      => EntityKind::Hashtag,
            "mention"                      => EntityKind::Mention,
            "emoji"                        => EntityKind::Emoji,
            other => return Err(DslError::Validation(format!(
                "unknown entity kind: {} (use email|phone|url|currency|ip_address|credit_card|iban|date_iso|hashtag|mention|emoji)",
                other
            ))),
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
                return Err(DslError::Validation(format!("unsupported language code: {} (use en|es|ca|eu)", c)));
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
    // ---------- v3 predicates ----------
    if let Some(n) = p.word_contains_any {
        if n.is_empty() {
            return Err(DslError::Validation("word_contains_any requires at least one needle".into()));
        }
        let needles = if cs { n } else { n.into_iter().map(|s| s.to_lowercase()).collect() };
        variants.push(Predicate::WordContainsAny { needles, case_sensitive: cs });
    }
    if let Some(prefixes) = p.starts_with_any {
        if prefixes.is_empty() {
            return Err(DslError::Validation("starts_with_any requires at least one prefix".into()));
        }
        let prefixes = if cs { prefixes } else { prefixes.into_iter().map(|s| s.to_lowercase()).collect() };
        variants.push(Predicate::StartsWithAny { prefixes, case_sensitive: cs });
    }
    if let Some(suffixes) = p.ends_with_any {
        if suffixes.is_empty() {
            return Err(DslError::Validation("ends_with_any requires at least one suffix".into()));
        }
        let suffixes = if cs { suffixes } else { suffixes.into_iter().map(|s| s.to_lowercase()).collect() };
        variants.push(Predicate::EndsWithAny { suffixes, case_sensitive: cs });
    }
    if let Some(r) = p.sentences {
        if r.min.is_none() && r.max.is_none() {
            return Err(DslError::Validation("sentences requires at least one of min/max".into()));
        }
        variants.push(Predicate::SentenceCount { min: r.min, max: r.max });
    }
    if let Some(r) = p.chars {
        if r.min.is_none() && r.max.is_none() {
            return Err(DslError::Validation("chars requires at least one of min/max".into()));
        }
        variants.push(Predicate::CharCount { min: r.min, max: r.max });
    }
    if let Some(r) = p.lines {
        if r.min.is_none() && r.max.is_none() {
            return Err(DslError::Validation("lines requires at least one of min/max".into()));
        }
        variants.push(Predicate::LineCount { min: r.min, max: r.max });
    }
    if let Some(r) = p.digit_ratio_above {
        if !(0.0..=1.0).contains(&r.min_ratio) {
            return Err(DslError::Validation(format!("digit_ratio_above min_ratio must be in [0,1], got {}", r.min_ratio)));
        }
        variants.push(Predicate::DigitRatioAbove { min_ratio: r.min_ratio });
    }
    if let Some(r) = p.punctuation_ratio_above {
        if !(0.0..=1.0).contains(&r.min_ratio) {
            return Err(DslError::Validation(format!("punctuation_ratio_above min_ratio must be in [0,1], got {}", r.min_ratio)));
        }
        variants.push(Predicate::PunctuationRatioAbove { min_ratio: r.min_ratio });
    }
    if let Some(r) = p.repeated_char_run {
        if r.min_run < 2 {
            return Err(DslError::Validation("repeated_char_run min_run must be >= 2".into()));
        }
        variants.push(Predicate::RepeatedCharRun { min_run: r.min_run });
    }
    if let Some(r) = p.repeated_token {
        if r.min_count < 2 {
            return Err(DslError::Validation("repeated_token min_count must be >= 2".into()));
        }
        variants.push(Predicate::RepeatedToken { min_count: r.min_count });
    }
    if let Some(r) = p.type_token_ratio_below {
        if !(0.0..=1.0).contains(&r.max_ratio) {
            return Err(DslError::Validation(format!("type_token_ratio_below max_ratio must be in [0,1], got {}", r.max_ratio)));
        }
        variants.push(Predicate::TypeTokenRatioBelow { max_ratio: r.max_ratio });
    }
    if p.has_invisible_chars {
        variants.push(Predicate::HasInvisibleChars);
    }
    if p.has_mixed_script_token {
        variants.push(Predicate::HasMixedScriptToken);
    }
    if let Some(scripts) = p.script_is {
        if scripts.is_empty() {
            return Err(DslError::Validation("script_is requires at least one script".into()));
        }
        for s in &scripts {
            if !is_known_script(s) {
                return Err(DslError::Validation(format!(
                    "unknown script: {} (use latin|cyrillic|greek|han|hiragana|katakana|hangul|arabic|hebrew|devanagari|thai)",
                    s
                )));
            }
        }
        variants.push(Predicate::ScriptIs { scripts });
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
                return Err(DslError::Validation(format!("unsupported language: {} (use en|es|ca|eu)", code)));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn err(src: &str) -> String {
        match parse(src) {
            Ok(_) => panic!("expected parse error, got Ok for:\n{}", src),
            Err(e) => e.to_string(),
        }
    }

    #[test]
    fn parse_minimal_succeeds() {
        let src = r#"
name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
"#;
        let p = parse(src).expect("parse ok");
        assert_eq!(p.name, "t");
        assert_eq!(p.rules.len(), 1);
        assert_eq!(p.rules[0].verdict.confidence, 1.0);
        assert_eq!(p.default.classify, "ok");
    }

    #[test]
    fn pack_name_required() {
        let msg = err(r#"
name: ""
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
"#);
        assert!(msg.contains("pack name is empty"), "got: {msg}");
    }

    #[test]
    fn duplicate_rule_id_rejected() {
        let msg = err(r#"
name: t
rules:
  - id: dup
    when: { contains_any: ["x"] }
    classify: a
  - id: dup
    when: { contains_any: ["y"] }
    classify: b
"#);
        assert!(msg.contains("duplicate rule id"), "got: {msg}");
        assert!(msg.contains("dup"), "got: {msg}");
    }

    #[test]
    fn then_to_unknown_rule_rejected() {
        let msg = err(r#"
name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
    then: ["nope"]
"#);
        assert!(msg.contains("chains to unknown rule"), "got: {msg}");
        assert!(msg.contains("nope"), "got: {msg}");
    }

    #[test]
    fn then_self_reference_rejected() {
        let msg = err(r#"
name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
    then: ["a"]
"#);
        assert!(msg.contains("chains to itself"), "got: {msg}");
    }

    #[test]
    fn invalid_regex_rejected_with_pattern_in_message() {
        let msg = err(r#"
name: t
rules:
  - id: bad
    when: { regex: "(unclosed" }
    classify: hit
"#);
        assert!(msg.contains("regex pattern"), "got: {msg}");
        assert!(msg.contains("(unclosed"), "got: {msg}");
        // Rule id should be threaded through.
        assert!(msg.contains("\"bad\""), "got: {msg}");
    }

    #[test]
    fn rule_confidence_out_of_range_rejected() {
        let msg = err(r#"
name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
    confidence: 1.5
"#);
        assert!(msg.contains("confidence 1.5"), "got: {msg}");
        assert!(msg.contains("out of [0,1]"), "got: {msg}");
    }

    #[test]
    fn default_confidence_out_of_range_rejected() {
        let msg = err(r#"
name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
default:
  classify: ok
  confidence: -0.1
"#);
        assert!(msg.contains("default confidence"), "got: {msg}");
        assert!(msg.contains("out of [0,1]"), "got: {msg}");
    }

    #[test]
    fn empty_classify_rejected() {
        let msg = err(r#"
name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: ""
"#);
        assert!(msg.contains("classify is empty"), "got: {msg}");
    }

    #[test]
    fn parse_error_has_line_and_snippet() {
        // intentional malformed YAML — unbalanced bracket
        let msg = err("name: t\nrules: [\n");
        assert!(msg.contains("parse error"), "got: {msg}");
        // line number should appear when available
        assert!(msg.contains("line"), "got: {msg}");
    }
}
