use crate::ir::{EntityKind, Predicate, Program, Rule, Verdict};
use crate::semantic;
use regex::{Regex, RegexBuilder};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use rustc_hash::FxHashMap;

#[derive(Debug, Serialize)]
pub struct EvalResult {
    pub classification: String,
    pub confidence: f32,
    pub triggered: Vec<TriggeredRule>,
    pub explanations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TriggeredRule {
    pub id: String,
    pub classification: String,
    pub confidence: f32,
    pub explanation: Option<String>,
}

/// Evaluate `program` against `input`. Triggered rules are collected (with
/// chained `then` rules followed transitively, cycle-safe). The triggered
/// rule with the highest confidence wins; if none trigger, the program's
/// `default` verdict is used.
pub fn evaluate(program: &Program, input: &str) -> EvalResult {
    let ctx = Ctx::new(input);
    let mut triggered: Vec<TriggeredRule> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    for rule in &program.rules {
        eval_rule(rule, &program.rules, &ctx, &mut triggered, &mut visited);
    }

    let winner: &Verdict = triggered
        .iter()
        .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
        .and_then(|t| program.rules.iter().find(|r| r.id == t.id).map(|r| &r.verdict))
        .unwrap_or(&program.default);

    let explanations = triggered.iter().filter_map(|t| t.explanation.clone()).collect();

    EvalResult {
        classification: winner.classify.clone(),
        confidence: winner.confidence,
        triggered,
        explanations,
    }
}

fn eval_rule(
    rule: &Rule,
    rules: &[Rule],
    ctx: &Ctx,
    out: &mut Vec<TriggeredRule>,
    visited: &mut HashSet<String>,
) {
    if !visited.insert(rule.id.clone()) { return; } // cycle / duplicate guard
    if !eval_predicate(&rule.when, ctx) { return; }
    out.push(TriggeredRule {
        id: rule.id.clone(),
        classification: rule.verdict.classify.clone(),
        confidence: rule.verdict.confidence,
        explanation: rule.verdict.explanation.clone(),
    });
    for chained_id in &rule.then {
        if let Some(next) = rules.iter().find(|r| r.id == *chained_id) {
            eval_rule(next, rules, ctx, out, visited);
        }
    }
}

/// Per-evaluation context: caches anything expensive to recompute, like
/// compiled regexes for the input. Lives only for the duration of one
/// `evaluate` call.
struct Ctx<'a> {
    input: &'a str,
    lower: String,
}

impl<'a> Ctx<'a> {
    fn new(input: &'a str) -> Self {
        Ctx { input, lower: input.to_lowercase() }
    }

    fn lower(&self) -> &str {
        &self.lower
    }

    fn regex(pattern: &str, case_sensitive: bool) -> Option<Regex> {
        regex_cache().get(pattern, case_sensitive)
    }
}

thread_local! {
    static GLOBAL_REGEX_CACHE: RefCell<FxHashMap<(String, bool), Option<Regex>>> = RefCell::new(FxHashMap::default());
}

fn regex_cache() -> RegexCache {
    RegexCache
}

struct RegexCache;

impl RegexCache {
    fn get(&self, pattern: &str, case_sensitive: bool) -> Option<Regex> {
        GLOBAL_REGEX_CACHE.with(|cell| {
            let mut cache = cell.borrow_mut();
            let key = (pattern.to_string(), case_sensitive);
            cache.entry(key).or_insert_with(|| {
                let mut builder = RegexBuilder::new(pattern);
                builder.case_insensitive(!case_sensitive);
                // For ASCII-only patterns without Unicode character classes,
                // disable Unicode mode for faster byte-oriented matching.
                if is_ascii_only_regex(pattern) {
                    builder.unicode(false);
                }
                builder.build().ok()
            }).clone()
        })
    }
}

/// Conservative heuristic: pattern is ASCII-only and contains no \p{…}, \P{…},
/// or explicit (?u) / (?-u) flag overrides. Safe to build with unicode(false).
fn is_ascii_only_regex(pattern: &str) -> bool {
    if !pattern.is_ascii() {
        return false;
    }
    // Reject explicit unicode flag directives and Unicode property escapes.
    if pattern.contains("(?u)") || pattern.contains("(?-u)") {
        return false;
    }
    // Quick scan for \p{ or \P{ — these require Unicode mode to work correctly.
    let bytes = pattern.as_bytes();
    for w in bytes.windows(3) {
        if w[0] == b'\\' && (w[1] == b'p' || w[1] == b'P') && w[2] == b'{' {
            return false;
        }
    }
    true
}

fn eval_predicate(p: &Predicate, ctx: &Ctx) -> bool {
    let input = ctx.input;
    match p {
        Predicate::ContainsAny { needles, case_sensitive } => {
            needles.iter().any(|n| contains_ctx(ctx, n, *case_sensitive))
        }
        Predicate::ContainsAll { needles, case_sensitive } => {
            needles.iter().all(|n| contains_ctx(ctx, n, *case_sensitive))
        }
        Predicate::NotContainsAny { needles, case_sensitive } => {
            !needles.iter().any(|n| contains_ctx(ctx, n, *case_sensitive))
        }
        Predicate::Regex { pattern, case_sensitive } => {
            Ctx::regex(pattern, *case_sensitive).map_or(false, |re| re.is_match(input))
        }
        Predicate::MinLength { tokens } => token_count(input) >= *tokens as usize,
        Predicate::MaxLength { tokens } => token_count(input) <= *tokens as usize,
        Predicate::All(items) => items.iter().all(|q| eval_predicate(q, ctx)),
        Predicate::Any(items) => items.iter().any(|q| eval_predicate(q, ctx)),
        Predicate::Not(inner) => !eval_predicate(inner, ctx),
        Predicate::Always => true,

        Predicate::HasSection { titles } => has_section(input, titles),
        Predicate::HasEntity { kind, min_count } => count_entities(input, *kind) >= *min_count as usize,
        Predicate::ParagraphCount { min, max } => {
            let n = paragraph_count(input);
            min.map_or(true, |m| n >= m as usize) && max.map_or(true, |m| n <= m as usize)
        }
        Predicate::MaxWordsPerSentence { max } => max_words_per_sentence(input) <= *max as usize,
        Predicate::LanguageIs { codes } => {
            let detected = crate::text::detect_language(input).code();
            codes.iter().any(|c| c == detected)
        }

        Predicate::HasUrlToDomain { domains, allow_subdomains } => {
            url_hosts(input).iter().any(|host| {
                domains.iter().any(|d| {
                    if host == d { return true; }
                    *allow_subdomains && host.ends_with(&format!(".{d}"))
                })
            })
        }
        Predicate::MostlyUppercase { min_ratio } => {
            let (mut letters, mut upper) = (0u32, 0u32);
            for c in input.chars() {
                if c.is_alphabetic() {
                    letters += 1;
                    if c.is_uppercase() { upper += 1; }
                }
            }
            if letters == 0 { false } else { (upper as f32) / (letters as f32) >= *min_ratio }
        }
        Predicate::TokenEntropyAbove { min_bits, min_token_len } => {
            input.split_whitespace().any(|tok| {
                let len = tok.chars().count();
                if (len as u32) < *min_token_len { return false; }
                shannon_entropy(tok) >= *min_bits
            })
        }

        Predicate::SemanticMatch { examples, threshold, extra_synonyms, language } => {
            let lang = language
                .as_deref()
                .and_then(crate::text::Language::from_code)
                .unwrap_or_else(|| crate::text::detect_language(input));

            let extra_idx = if extra_synonyms.is_empty() {
                None
            } else {
                let mut json = String::from("{");
                for (i, (canonical, syns)) in extra_synonyms.iter().enumerate() {
                    if i > 0 { json.push(','); }
                    json.push_str(&format!("{}:{}",
                        serde_json::to_string(canonical).unwrap(),
                        serde_json::to_string(syns).unwrap()));
                }
                json.push('}');
                Some(semantic::SynonymIndex::from_json_for(&json, lang))
            };

            examples.iter().any(|ex| {
                semantic::similarity_lang(input, ex, lang, extra_idx.as_ref()) >= *threshold
            })
        }
    }
}

// ---------- predicate helpers ----------

fn contains_ctx(ctx: &Ctx, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        ctx.input.contains(needle)
    } else {
        ctx.lower().contains(needle)
    }
}

fn token_count(s: &str) -> usize {
    s.split_whitespace().count()
}

fn paragraph_count(s: &str) -> usize {
    s.split("\n\n").map(str::trim).filter(|p| !p.is_empty()).count().max(if s.trim().is_empty() { 0 } else { 1 })
}

fn max_words_per_sentence(s: &str) -> usize {
    s.split(|c: char| matches!(c, '.' | '!' | '?'))
        .map(|sent| sent.split_whitespace().count())
        .max()
        .unwrap_or(0)
}

/// Recognises markdown `#`-style headings and `<h1>..<h6>` HTML headings.
/// Title comparison is case-insensitive and trims whitespace.
fn has_section(input: &str, titles: &[String]) -> bool {
    let wants: Vec<String> = titles.iter().map(|t| t.trim().to_lowercase()).collect();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            // Strip leading '#' chars and a single space.
            let title = trimmed.trim_start_matches('#').trim().to_lowercase();
            if wants.iter().any(|w| w == &title) { return true; }
        }
    }
    // Cheap HTML heading match: `<h1>Title</h1>` etc.
    let lower = input.to_lowercase();
    for level in 1..=6 {
        let open = format!("<h{level}");
        let close = format!("</h{level}>");
        let mut start = 0usize;
        while let Some(idx) = lower[start..].find(&open) {
            let abs = start + idx;
            // Skip past the '>' that closes the opening tag.
            let after_open = &lower[abs..];
            if let Some(gt) = after_open.find('>') {
                let body_start = abs + gt + 1;
                if let Some(end_idx) = lower[body_start..].find(&close) {
                    let body = &lower[body_start..body_start + end_idx];
                    let body_clean = strip_html_tags(body).trim().to_string();
                    if wants.iter().any(|w| w == &body_clean) { return true; }
                    start = body_start + end_idx + close.len();
                    continue;
                }
            }
            break;
        }
    }
    false
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Extract hostnames (lowercased) from `http(s)://` URLs in the input.
/// Strips userinfo, port, and trailing path. Robust enough for predicate use,
/// not a general URL parser.
fn url_hosts(input: &str) -> Vec<String> {
    use std::sync::OnceLock;
    static URL: OnceLock<Regex> = OnceLock::new();
    let re = URL.get_or_init(|| Regex::new(r"(?i)\bhttps?://([^\s/?#]+)").unwrap());
    re.captures_iter(input)
        .filter_map(|c| c.get(1))
        .map(|m| {
            let mut host = m.as_str().to_lowercase();
            if let Some(at) = host.rfind('@') { host = host[at + 1..].to_string(); }
            if let Some(colon) = host.find(':') { host.truncate(colon); }
            host
        })
        .collect()
}

/// Shannon entropy in bits over the empirical char distribution of `s`.
/// Pure ASCII strings of length 16 with full alphanum diversity sit ~5 bits.
fn shannon_entropy(s: &str) -> f32 {
    let mut counts: HashMap<char, u32> = HashMap::new();
    let mut total = 0u32;
    for c in s.chars() { *counts.entry(c).or_insert(0) += 1; total += 1; }
    if total == 0 { return 0.0; }
    let n = total as f32;
    counts.values().map(|&c| {
        let p = c as f32 / n;
        -p * p.log2()
    }).sum()
}

fn count_entities(input: &str, kind: EntityKind) -> usize {
    use std::sync::OnceLock;
    static EMAIL: OnceLock<Regex> = OnceLock::new();
    static PHONE: OnceLock<Regex> = OnceLock::new();
    static URL:   OnceLock<Regex> = OnceLock::new();
    static CURR:  OnceLock<Regex> = OnceLock::new();
    let re = match kind {
        EntityKind::Email    => EMAIL.get_or_init(|| Regex::new(r"(?i)\b[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}\b").unwrap()),
        EntityKind::Phone    => PHONE.get_or_init(|| Regex::new(r"\b(?:\+?\d{1,3}[\s\-.]?)?(?:\(?\d{2,4}\)?[\s\-.]?){2,4}\d{2,4}\b").unwrap()),
        EntityKind::Url      => URL.get_or_init(|| Regex::new(r"(?i)\bhttps?://[a-z0-9.\-]+(?:/[^\s]*)?").unwrap()),
        EntityKind::Currency => CURR.get_or_init(|| Regex::new(r"(?:[\$£€¥]\s?\d{1,3}(?:[,.]\d{3})*(?:\.\d+)?|\b\d+(?:[.,]\d+)?\s?(?:USD|EUR|GBP|JPY)\b)").unwrap()),
    };
    re.find_iter(input).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_yaml;

    const SAMPLE: &str = r#"
name: forbidden_phrases
rules:
  - id: no_guarantee
    when:
      contains_any: ["guaranteed refund", "100% refund"]
    classify: review_required
    confidence: 0.95
    explanation: "Forbidden refund guarantee"
  - id: missing_disclaimer
    when:
      not_contains_any: ["This is not financial advice"]
    classify: review_required
    confidence: 0.8
default:
  classify: ok
  confidence: 1.0
"#;

    #[test]
    fn triggers_on_forbidden_phrase() {
        let program = compile_yaml(SAMPLE).unwrap();
        let r = evaluate(&program, "We offer a guaranteed refund. This is not financial advice");
        assert_eq!(r.classification, "review_required");
        assert!(r.triggered.iter().any(|t| t.id == "no_guarantee"));
    }

    #[test]
    fn defaults_to_ok() {
        let program = compile_yaml(SAMPLE).unwrap();
        let r = evaluate(&program, "Hello there. This is not financial advice");
        assert_eq!(r.classification, "ok");
    }

    #[test]
    fn highest_confidence_wins() {
        let program = compile_yaml(SAMPLE).unwrap();
        let r = evaluate(&program, "guaranteed refund right now");
        assert_eq!(r.triggered.len(), 2);
        assert_eq!(r.classification, "review_required");
        assert!((r.confidence - 0.95).abs() < 1e-6);
    }

    #[test]
    fn regex_predicate_works() {
        let yaml = r#"
name: r
rules:
  - id: phone
    when:
      regex: "\\b\\d{3}-\\d{4}\\b"
      case_sensitive: true
    classify: pii
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "call 555-1234").classification, "pii");
        assert_eq!(evaluate(&p, "no number here").classification, "ok");
    }

    #[test]
    fn entity_predicate_email() {
        let yaml = r#"
name: pii
rules:
  - id: has_email
    when: { has_entity: { kind: email } }
    classify: pii
    confidence: 0.95
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "ping me at alice@example.com").classification, "pii");
        assert_eq!(evaluate(&p, "no contact info here").classification, "ok");
    }

    #[test]
    fn structural_section_predicate() {
        let yaml = r#"
name: contracts
rules:
  - id: has_termination
    when:
      has_section: ["Termination", "Cancellation"]
    classify: ok_contract
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        let md = "# Parties\nFoo and bar.\n\n## Termination\nEither party may...";
        assert_eq!(evaluate(&p, md).classification, "ok_contract");
        let html = "<p>x</p><h2>Cancellation</h2><p>y</p>";
        assert_eq!(evaluate(&p, html).classification, "ok_contract");
    }

    #[test]
    fn rule_chaining_then_field() {
        let yaml = r#"
name: chain
rules:
  - id: trigger
    when: { contains_any: ["alpha"] }
    classify: a
    confidence: 0.5
    then: ["next"]
  - id: next
    when: { always: true }
    classify: b
    confidence: 0.9
default: { classify: ok, confidence: 1.0 }
"#;
        let p = compile_yaml(yaml).unwrap();
        let r = evaluate(&p, "alpha");
        // Both fired; "next" wins on confidence.
        assert_eq!(r.classification, "b");
        assert_eq!(r.triggered.len(), 2);
    }

    #[test]
    fn semantic_match_triggers_on_synonyms() {
        let yaml = r#"
name: cancellation_detector
rules:
  - id: mentions_cancellation
    when:
      semantic_match:
        examples:
          - "the user wants to cancel their agreement"
        threshold: 0.4
    classify: cancellation_intent
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        let r = evaluate(&p, "Please terminate my contract immediately");
        assert_eq!(r.classification, "cancellation_intent");
    }

    #[test]
    fn semantic_match_does_not_trigger_unrelated() {
        let yaml = r#"
name: cancellation_detector
rules:
  - id: mentions_cancellation
    when:
      semantic_match:
        examples: ["the user wants to cancel their agreement"]
        threshold: 0.4
    classify: cancellation_intent
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        let r = evaluate(&p, "I love your product, the colors are great");
        assert_eq!(r.classification, "ok");
    }

    #[test]
    fn roundtrip_encode_decode() {
        let p = compile_yaml(SAMPLE).unwrap();
        let bytes = crate::encode(&p);
        let p2 = crate::decode(&bytes).unwrap();
        assert_eq!(p.name, p2.name);
        assert_eq!(p.rules.len(), p2.rules.len());
    }

    #[test]
    fn decode_rejects_unknown_magic() {
        let bad = vec![0u8; 32];
        assert!(matches!(crate::decode(&bad), Err(crate::DecodeError::BadMagic)));
    }

    #[test]
    fn has_url_to_domain_matches_subdomains() {
        let yaml = r#"
name: phishy
rules:
  - id: external_link
    when:
      not:
        has_url_to_domain:
          domains: ["example.com", "trusted.org"]
          allow_subdomains: true
    classify: blocked
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        // URL within allowlist subdomain → no trigger.
        assert_eq!(evaluate(&p, "click https://docs.example.com/page").classification, "ok");
        // URL outside allowlist → triggers.
        assert_eq!(evaluate(&p, "click https://evil.com/login").classification, "blocked");
    }

    #[test]
    fn has_url_to_domain_strips_userinfo_and_port() {
        let yaml = r#"
name: u
rules:
  - id: hits_corp
    when:
      has_url_to_domain:
        domains: ["corp.example"]
        allow_subdomains: false
    classify: corp
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "go https://user:pw@corp.example:8443/x").classification, "corp");
        // Subdomain when allow_subdomains=false → does not trigger.
        assert_eq!(evaluate(&p, "go https://app.corp.example/").classification, "ok");
    }

    #[test]
    fn mostly_uppercase_triggers_on_shouting() {
        let yaml = r#"
name: shout
rules:
  - id: yelling
    when: { mostly_uppercase: { min_ratio: 0.7 } }
    classify: review_required
    confidence: 0.7
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "BUY NOW LIMITED OFFER").classification, "review_required");
        assert_eq!(evaluate(&p, "Hello, this is a normal sentence.").classification, "ok");
        // No letters at all → never trigger.
        assert_eq!(evaluate(&p, "12345 !!!").classification, "ok");
    }

    #[test]
    fn token_entropy_finds_random_secret() {
        let yaml = r#"
name: ent
rules:
  - id: high_entropy
    when:
      token_entropy_above:
        min_bits: 4.0
        min_token_len: 16
    classify: contains_secret
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        // High-entropy 32-char token (no vendor prefix, so a regex pack would miss).
        assert_eq!(
            evaluate(&p, "deploy with token aZ9bX2qW7eR4tY6uI8oP3sD5fG1hJ0kL").classification,
            "contains_secret"
        );
        // Plain English prose: no token has high enough entropy.
        assert_eq!(
            evaluate(&p, "the quick brown fox jumps over the lazy dog repeatedly").classification,
            "ok"
        );
    }
}
