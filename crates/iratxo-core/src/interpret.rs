use crate::ir::{EntityKind, Predicate, Program, Rule, Verdict};
use crate::semantic;
use regex::{Regex, RegexBuilder};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash as _, Hasher};
use rustc_hash::{FxHashMap, FxHasher};

/// Detailed metrics for a single `evaluate` call.
#[derive(Debug, Default, Clone, Serialize)]
pub struct EvalMetrics {
    pub rule_evals: u64,
    pub predicate_evals: u64,
    pub predicate_true: u64,
    pub predicate_false: u64,
    pub regex_cache_hits: u64,
    pub regex_cache_misses: u64,
    pub tokenize_calls: u64,
    pub tokens_produced: u64,
    pub stem_calls: u64,
    pub semantic_similarity_calls: u64,
    pub entity_detection_calls: u64,
    pub url_extract_calls: u64,
    pub section_scan_calls: u64,
    pub chain_traversals: u64,
    pub triggered_rules: u64,
    pub all_short_circuits: u64,
    pub any_short_circuits: u64,
    pub not_short_circuits: u64,
    pub max_chain_depth: u64,
    pub language_detect_calls: u64,
    pub lower_allocations: u64,
    pub lower_bytes: u64,
}

thread_local! {
    static METRICS: RefCell<Option<EvalMetrics>> = RefCell::new(None);
    static METRICS_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
    /// Cross-evaluate cache for entity counts keyed by (input_hash, kind, min_count).
    /// Capped at 256 entries to avoid unbounded growth.
    static ENTITY_COUNT_CACHE: RefCell<FxHashMap<(u64, EntityKind, u32), usize>> = RefCell::new(FxHashMap::default());
    /// Cross-evaluate cache for semantic example embeddings keyed by
    /// (text_hash, language, extra_hash).
    static SEMANTIC_EXAMPLE_CACHE: RefCell<FxHashMap<(u64, crate::text::Language, u64), [f32; 256]>> = RefCell::new(FxHashMap::default());
}

#[inline]
fn with_metrics<F: FnOnce(&mut EvalMetrics)>(f: F) {
    if !METRICS_ENABLED.with(|c| c.get()) { return; }
    METRICS.with(|cell| {
        if let Some(ref mut m) = *cell.borrow_mut() {
            f(m);
        }
    });
}

/// Evaluate `program` against `input` and return both the result and a
/// detailed metrics snapshot. Zero-cost when not called — the normal
/// `evaluate` path never touches the metrics thread-local.
pub fn evaluate_with_metrics(program: &Program, input: &str) -> (EvalResult, EvalMetrics) {
    METRICS_ENABLED.with(|c| c.set(true));
    METRICS.with(|cell| {
        *cell.borrow_mut() = Some(EvalMetrics::default());
    });
    let result = evaluate(program, input);
    let metrics = METRICS.with(|cell| cell.borrow_mut().take().unwrap_or_default());
    METRICS_ENABLED.with(|c| c.set(false));
    (result, metrics)
}

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

/// Zero-copy evaluation result that borrows strings from `program`.
/// Useful for Wasm and other contexts where the result is serialized
/// immediately and string clones are pure overhead.
#[derive(Debug, Serialize)]
pub struct EvalResultRef<'a> {
    pub classification: &'a str,
    pub confidence: f32,
    pub triggered: Vec<TriggeredRuleRef<'a>>,
    pub explanations: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct TriggeredRuleRef<'a> {
    pub id: &'a str,
    pub classification: &'a str,
    pub confidence: f32,
    pub explanation: Option<&'a str>,
}

/// Evaluate `program` against `input`. Triggered rules are collected (with
/// chained `then` rules followed transitively, cycle-safe). The triggered
/// rule with the highest confidence wins; if none trigger, the program's
/// `default` verdict is used.
pub fn evaluate(program: &Program, input: &str) -> EvalResult {
    let r = evaluate_ref(program, input);
    EvalResult {
        classification: r.classification.to_string(),
        confidence: r.confidence,
        triggered: r.triggered.into_iter().map(|t| TriggeredRule {
            id: t.id.to_string(),
            classification: t.classification.to_string(),
            confidence: t.confidence,
            explanation: t.explanation.map(|s| s.to_string()),
        }).collect(),
        explanations: r.explanations.into_iter().map(|s| s.to_string()).collect(),
    }
}

/// Zero-copy variant of [`evaluate`]. Returns borrowed strings referencing
/// the compiled `program`. The caller must ensure `program` outlives the
/// returned value (true in the typical compile-once-evaluate-many pattern).
pub fn evaluate_ref<'a>(program: &'a Program, input: &str) -> EvalResultRef<'a> {
    let ctx = Ctx::new(input);
    let mut triggered: Vec<TriggeredRuleRef<'a>> = Vec::new();
    let mut visited: HashSet<&'a str> = HashSet::new();
    for rule in &program.rules {
        if program.chained_targets.contains(&rule.id) { continue; }
        eval_rule_ref(rule, &program.rules, &ctx, &mut triggered, &mut visited, 0);
    }

    let winner = triggered
        .iter()
        .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal));

    let classification = winner.map(|t| t.classification).unwrap_or(&program.default.classify);
    let confidence = winner.map(|t| t.confidence).unwrap_or(program.default.confidence);
    let explanations: Vec<&'a str> = triggered.iter().filter_map(|t| t.explanation).collect();

    EvalResultRef {
        classification,
        confidence,
        triggered,
        explanations,
    }
}

#[inline]
fn eval_rule<'a>(
    rule: &'a Rule,
    rules: &'a [Rule],
    ctx: &Ctx,
    out: &mut Vec<TriggeredRule>,
    visited: &mut HashSet<&'a str>,
    depth: u64,
) {
    with_metrics(|m| {
        m.rule_evals += 1;
        if depth > m.max_chain_depth {
            m.max_chain_depth = depth;
        }
    });
    if !visited.insert(rule.id.as_str()) { return; }
    if !eval_predicate(&rule.when, ctx) { return; }
    with_metrics(|m| m.triggered_rules += 1);
    out.push(TriggeredRule {
        id: rule.id.clone(),
        classification: rule.verdict.classify.clone(),
        confidence: rule.verdict.confidence,
        explanation: rule.verdict.explanation.clone(),
    });
    for chained_id in &rule.then {
        if let Some(next) = rules.iter().find(|r| r.id == *chained_id) {
            with_metrics(|m| m.chain_traversals += 1);
            eval_rule(next, rules, ctx, out, visited, depth + 1);
        }
    }
}

#[inline]
fn eval_rule_ref<'a>(
    rule: &'a Rule,
    rules: &'a [Rule],
    ctx: &Ctx,
    out: &mut Vec<TriggeredRuleRef<'a>>,
    visited: &mut HashSet<&'a str>,
    depth: u64,
) {
    with_metrics(|m| {
        m.rule_evals += 1;
        if depth > m.max_chain_depth {
            m.max_chain_depth = depth;
        }
    });
    if !visited.insert(rule.id.as_str()) { return; }
    if !eval_predicate(&rule.when, ctx) { return; }
    with_metrics(|m| m.triggered_rules += 1);
    out.push(TriggeredRuleRef {
        id: rule.id.as_str(),
        classification: rule.verdict.classify.as_str(),
        confidence: rule.verdict.confidence,
        explanation: rule.verdict.explanation.as_deref(),
    });
    for chained_id in &rule.then {
        if let Some(next) = rules.iter().find(|r| r.id == *chained_id) {
            with_metrics(|m| m.chain_traversals += 1);
            eval_rule_ref(next, rules, ctx, out, visited, depth + 1);
        }
    }
}

/// Per-evaluation context: caches anything expensive to recompute, like
/// compiled regexes for the input. Lives only for the duration of one
/// `evaluate` call.
struct Ctx<'a> {
    input: &'a str,
    lower: String,
    input_hash: u64,
    url_hosts: RefCell<Option<Vec<String>>>,
    lang: RefCell<Option<crate::text::Language>>,
    semantic_embed: RefCell<HashMap<(crate::text::Language, u64), [f32; 256]>>,
}

impl<'a> Ctx<'a> {
    fn new(input: &'a str) -> Self {
        let lower = if input.is_ascii() {
            input.to_ascii_lowercase()
        } else {
            input.to_lowercase()
        };
        with_metrics(|m| {
            m.lower_allocations += 1;
            m.lower_bytes += lower.len() as u64;
        });
        let mut hasher = FxHasher::default();
        input.hash(&mut hasher);
        let input_hash = hasher.finish();
        Ctx {
            input,
            lower,
            input_hash,
            url_hosts: RefCell::new(None),
            lang: RefCell::new(None),
            semantic_embed: RefCell::new(HashMap::new()),
        }
    }

    fn lower(&self) -> &str {
        &self.lower
    }

    fn regex(pattern: &str, case_sensitive: bool) -> Option<Regex> {
        regex_cache().get(pattern, case_sensitive)
    }

    fn detect_language(&self) -> crate::text::Language {
        let mut cache = self.lang.borrow_mut();
        *cache.get_or_insert_with(|| {
            with_metrics(|m| m.language_detect_calls += 1);
            crate::text::detect_language(self.input)
        })
    }

    fn semantic_embed(&self, lang: crate::text::Language, extra: Option<&semantic::SynonymIndex>, extra_hash: u64) -> [f32; 256] {
        let mut cache = self.semantic_embed.borrow_mut();
        let key = (lang, extra_hash);
        *cache.entry(key).or_insert_with(|| {
            semantic::embed_input(self.input, lang, extra)
        })
    }

    fn count_entities(&self, kind: EntityKind, min_count: u32) -> bool {
        let key = (self.input_hash, kind, min_count);
        let cached = ENTITY_COUNT_CACHE.with(|cell| cell.borrow().get(&key).copied());
        if let Some(count) = cached {
            return count >= min_count as usize;
        }
        with_metrics(|m| m.entity_detection_calls += 1);
        let count = count_entities_impl(self.input, kind, min_count);
        ENTITY_COUNT_CACHE.with(|cell| {
            let mut cache = cell.borrow_mut();
            cache.insert(key, count);
            // Simple cap: if exceeded, clear to avoid unbounded growth.
            if cache.len() > 256 {
                cache.clear();
            }
        });
        count >= min_count as usize
    }

    fn url_hosts(&self) -> Vec<String> {
        let mut cache = self.url_hosts.borrow_mut();
        cache.get_or_insert_with(|| {
            with_metrics(|m| m.url_extract_calls += 1);
            url_hosts_impl(self.input)
        }).clone()
    }
}

fn regex_cache_key(pattern: &str, case_sensitive: bool) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    pattern.hash(&mut hasher);
    case_sensitive.hash(&mut hasher);
    hasher.finish()
}

thread_local! {
    static GLOBAL_REGEX_CACHE: RefCell<FxHashMap<u64, Option<Regex>>> = RefCell::new(FxHashMap::default());
}

fn regex_cache() -> RegexCache {
    RegexCache
}

struct RegexCache;

impl RegexCache {
    fn get(&self, pattern: &str, case_sensitive: bool) -> Option<Regex> {
        let key = regex_cache_key(pattern, case_sensitive);
        GLOBAL_REGEX_CACHE.with(|cell| {
            let mut cache = cell.borrow_mut();
            let was_present = cache.contains_key(&key);
            let result = cache.entry(key).or_insert_with(|| {
                let mut builder = RegexBuilder::new(pattern);
                builder.case_insensitive(!case_sensitive);
                // For ASCII-only patterns without Unicode character classes,
                // disable Unicode mode for faster byte-oriented matching.
                if is_ascii_only_regex(pattern) {
                    builder.unicode(false);
                }
                builder.build().ok()
            }).clone();
            if was_present {
                with_metrics(|m| m.regex_cache_hits += 1);
            } else {
                with_metrics(|m| m.regex_cache_misses += 1);
            }
            result
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

#[inline]
fn eval_predicate(p: &Predicate, ctx: &Ctx) -> bool {
    with_metrics(|m| m.predicate_evals += 1);
    let input = ctx.input;
    let result = match p {
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
        Predicate::All(items) => {
            let r = items.iter().all(|q| eval_predicate(q, ctx));
            if !r { with_metrics(|m| m.all_short_circuits += 1); }
            r
        }
        Predicate::Any(items) => {
            let r = items.iter().any(|q| eval_predicate(q, ctx));
            if r { with_metrics(|m| m.any_short_circuits += 1); }
            r
        }
        Predicate::Not(inner) => {
            let r = !eval_predicate(inner, ctx);
            with_metrics(|m| m.not_short_circuits += 1);
            r
        }
        Predicate::Always => true,

        Predicate::HasSection { titles } => {
            with_metrics(|m| m.section_scan_calls += 1);
            has_section(ctx, titles)
        }
        Predicate::HasEntity { kind, min_count } => {
            ctx.count_entities(*kind, *min_count)
        }
        Predicate::ParagraphCount { min, max } => {
            let n = paragraph_count(input);
            min.map_or(true, |m| n >= m as usize) && max.map_or(true, |m| n <= m as usize)
        }
        Predicate::MaxWordsPerSentence { max } => max_words_per_sentence(input) <= *max as usize,
        Predicate::LanguageIs { codes } => {
            let detected = ctx.detect_language().code();
            codes.iter().any(|c| c == detected)
        }

        Predicate::HasUrlToDomain { domains, allow_subdomains } => {
            ctx.url_hosts().iter().any(|host| {
                domains.iter().any(|d| {
                    if host == d { return true; }
                    if !*allow_subdomains { return false; }
                    // Avoid format!(".{d}") allocation.
                    let host_bytes = host.as_bytes();
                    let d_bytes = d.as_bytes();
                    if host_bytes.len() <= d_bytes.len() + 1 { return false; }
                    let prefix = host_bytes.len() - d_bytes.len() - 1;
                    host_bytes[prefix] == b'.' && &host_bytes[prefix + 1..] == d_bytes
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

        Predicate::SemanticMatch(data) => {
            let lang = data.language
                .as_deref()
                .and_then(crate::text::Language::from_code)
                .unwrap_or_else(|| ctx.detect_language());

            let mut fallback_idx: Option<semantic::SynonymIndex> = None;
            let (extra_idx, hash) = if data.extra_synonyms.is_empty() {
                (None, 0u64)
            } else {
                let mut json = String::from("{");
                for (i, (canonical, syns)) in data.extra_synonyms.iter().enumerate() {
                    if i > 0 { json.push(','); }
                    json.push_str(&format!("{}:{}",
                        serde_json::to_string(canonical).unwrap(),
                        serde_json::to_string(syns).unwrap()));
                }
                json.push('}');
                let h = semantic::fnv1a64(json.as_bytes());
                fallback_idx = Some(semantic::SynonymIndex::from_json_for(&json, lang));
                (fallback_idx.as_ref(), h)
            };

            with_metrics(|m| m.semantic_similarity_calls += data.examples.len() as u64);
            let input_embed = ctx.semantic_embed(lang, extra_idx, hash);
            data.examples.iter().any(|ex| {
                let mut hasher = FxHasher::default();
                ex.as_bytes().hash(&mut hasher);
                let ex_hash = hasher.finish();
                let key = (ex_hash, lang, hash);
                let cached = SEMANTIC_EXAMPLE_CACHE.with(|cell| cell.borrow().get(&key).copied());
                let ex_embed = match cached {
                    Some(e) => e,
                    None => {
                        let e = semantic::embed_input(ex, lang, extra_idx);
                        SEMANTIC_EXAMPLE_CACHE.with(|cell| {
                            let mut cache = cell.borrow_mut();
                            cache.insert(key, e);
                            if cache.len() > 256 {
                                cache.clear();
                            }
                        });
                        e
                    }
                };
                semantic::cosine(&input_embed, &ex_embed) >= data.threshold
            })
        }

        // ---------- v3 heuristics ----------
        Predicate::WordContainsAny { needles, case_sensitive } => {
            let hay = if *case_sensitive { input } else { ctx.lower() };
            needles.iter().any(|n| word_contains(hay, n))
        }
        Predicate::StartsWithAny { prefixes, case_sensitive } => {
            let hay = if *case_sensitive { input.trim_start() } else {
                ctx.lower().trim_start_matches(char::is_whitespace)
            };
            prefixes.iter().any(|p| hay.starts_with(p.as_str()))
        }
        Predicate::EndsWithAny { suffixes, case_sensitive } => {
            let hay = if *case_sensitive { input.trim_end() } else {
                ctx.lower().trim_end_matches(char::is_whitespace)
            };
            suffixes.iter().any(|s| hay.ends_with(s.as_str()))
        }
        Predicate::SentenceCount { min, max } => {
            let n = sentence_count(input);
            min.map_or(true, |m| n >= m as usize) && max.map_or(true, |m| n <= m as usize)
        }
        Predicate::CharCount { min, max } => {
            let n = input.chars().count();
            min.map_or(true, |m| n >= m as usize) && max.map_or(true, |m| n <= m as usize)
        }
        Predicate::LineCount { min, max } => {
            let n = if input.is_empty() { 0 } else { input.lines().count() };
            min.map_or(true, |m| n >= m as usize) && max.map_or(true, |m| n <= m as usize)
        }
        Predicate::DigitRatioAbove { min_ratio } => {
            let (mut total, mut digits) = (0u32, 0u32);
            for c in input.chars() {
                if c.is_whitespace() { continue; }
                total += 1;
                if c.is_ascii_digit() { digits += 1; }
            }
            if total == 0 { false } else { (digits as f32) / (total as f32) >= *min_ratio }
        }
        Predicate::PunctuationRatioAbove { min_ratio } => {
            let (mut total, mut punct) = (0u32, 0u32);
            for c in input.chars() {
                if c.is_whitespace() { continue; }
                total += 1;
                if c.is_ascii_punctuation() { punct += 1; }
            }
            if total == 0 { false } else { (punct as f32) / (total as f32) >= *min_ratio }
        }
        Predicate::RepeatedCharRun { min_run } => {
            let mut prev: Option<char> = None;
            let mut run: u32 = 0;
            for c in input.chars() {
                if c.is_whitespace() {
                    prev = None;
                    run = 0;
                    continue;
                }
                if Some(c) == prev {
                    run += 1;
                    if run >= *min_run { return true; }
                } else {
                    prev = Some(c);
                    run = 1;
                }
            }
            false
        }
        Predicate::RepeatedToken { min_count } => {
            let mut counts: FxHashMap<&str, u32> = FxHashMap::default();
            let mut produced = 0u64;
            for tok in crate::text::tokenize_iter(ctx.lower()) {
                produced += 1;
                if tok.chars().count() < 2 { continue; }
                let entry = counts.entry(tok).or_insert(0);
                *entry += 1;
                if *entry >= *min_count {
                    with_metrics(|m| { m.tokenize_calls += 1; m.tokens_produced += produced; });
                    return true;
                }
            }
            with_metrics(|m| { m.tokenize_calls += 1; m.tokens_produced += produced; });
            false
        }
        Predicate::TypeTokenRatioBelow { max_ratio } => {
            let mut total = 0u64;
            let mut unique = rustc_hash::FxHashSet::default();
            for tok in crate::text::tokenize_iter(ctx.lower()) {
                total += 1;
                unique.insert(tok);
            }
            with_metrics(|m| { m.tokenize_calls += 1; m.tokens_produced += total; });
            if total == 0 { return false; }
            (unique.len() as f32 / total as f32) <= *max_ratio
        }
        Predicate::HasInvisibleChars => input.chars().any(is_invisible_char),
        Predicate::HasMixedScriptToken => {
            input.split_whitespace().any(|tok| token_uses_multiple_scripts(tok))
        }
        Predicate::ScriptIs { scripts } => {
            scripts.iter().any(|s| {
                input.chars().any(|c| char_in_script(c, s))
            })
        }
    };
    with_metrics(|m| {
        if result { m.predicate_true += 1; } else { m.predicate_false += 1; }
    });
    result
}

// ---------- predicate helpers ----------

#[inline]
fn contains_ctx(ctx: &Ctx, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        ctx.input.contains(needle)
    } else {
        ctx.lower().contains(needle)
    }
}

#[inline]
fn token_count(s: &str) -> usize {
    s.split_whitespace().count()
}

#[inline]
fn paragraph_count(s: &str) -> usize {
    s.split("\n\n").map(str::trim).filter(|p| !p.is_empty()).count().max(if s.trim().is_empty() { 0 } else { 1 })
}

#[inline]
fn max_words_per_sentence(s: &str) -> usize {
    s.split(|c: char| matches!(c, '.' | '!' | '?'))
        .map(|sent| sent.split_whitespace().count())
        .max()
        .unwrap_or(0)
}

/// Recognises markdown `#`-style headings and `<h1>..<h6>` HTML headings.
/// Title comparison is case-insensitive and trims whitespace.
/// Titles are expected to be pre-trimmed and pre-lowercased at compile time.
#[inline]
fn has_section(ctx: &Ctx, titles: &[String]) -> bool {
    for line in ctx.input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            // Strip leading '#' chars and a single space.
            let title = trimmed.trim_start_matches('#').trim().to_lowercase();
            if titles.iter().any(|w| *w == title) { return true; }
        }
    }
    // Cheap HTML heading match: `<h1>Title</h1>` etc.
    // Use ctx.lower() instead of re-lowercasing the whole input.
    const HTML_OPEN: [&str; 6] = ["<h1", "<h2", "<h3", "<h4", "<h5", "<h6"];
    const HTML_CLOSE: [&str; 6] = ["</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>"];
    let lower = ctx.lower();
    for level in 1..=6 {
        let open = HTML_OPEN[level - 1];
        let close = HTML_CLOSE[level - 1];
        let mut start = 0usize;
        while let Some(idx) = lower[start..].find(open) {
            let abs = start + idx;
            // Skip past the '>' that closes the opening tag.
            let after_open = &lower[abs..];
            if let Some(gt) = after_open.find('>') {
                let body_start = abs + gt + 1;
                if let Some(end_idx) = lower[body_start..].find(close) {
                    let body = &lower[body_start..body_start + end_idx];
                    let body_clean = strip_html_tags(body);
                    let body_trimmed = body_clean.trim();
                    if titles.iter().any(|w| *w == body_trimmed) { return true; }
                    start = body_start + end_idx + close.len();
                    continue;
                }
            }
            break;
        }
    }
    false
}

#[inline]
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
#[inline]
fn url_hosts_impl(input: &str) -> Vec<String> {
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
#[inline]
fn shannon_entropy(s: &str) -> f32 {
    if s.is_empty() { return 0.0; }
    // Fast path for ASCII-only strings: use a fixed-size array instead of HashMap.
    if s.is_ascii() {
        let mut counts = [0u32; 128];
        let mut total = 0u32;
        for b in s.as_bytes() {
            counts[*b as usize] += 1;
            total += 1;
        }
        let n = total as f32;
        return counts.iter().filter(|&&c| c > 0).map(|&c| {
            let p = c as f32 / n;
            -p * p.log2()
        }).sum();
    }
    let mut counts: HashMap<char, u32> = HashMap::new();
    let mut total = 0u32;
    for c in s.chars() { *counts.entry(c).or_insert(0) += 1; total += 1; }
    let n = total as f32;
    counts.values().map(|&c| {
        let p = c as f32 / n;
        -p * p.log2()
    }).sum()
}

/// Count regex matches, stopping as soon as `limit` is reached.
#[inline]
fn regex_count_early(re: &Regex, input: &str, limit: u32) -> usize {
    if limit == 0 { return 0; }
    let mut count = 0usize;
    for _ in re.find_iter(input) {
        count += 1;
        if count >= limit as usize { break; }
    }
    count
}

#[inline]
fn count_entities_impl(input: &str, kind: EntityKind, min_count: u32) -> usize {
    use std::sync::OnceLock;
    static EMAIL:   OnceLock<Regex> = OnceLock::new();
    static PHONE:   OnceLock<Regex> = OnceLock::new();
    static URL:     OnceLock<Regex> = OnceLock::new();
    static CURR:    OnceLock<Regex> = OnceLock::new();
    static IP:      OnceLock<Regex> = OnceLock::new();
    static CARD:    OnceLock<Regex> = OnceLock::new();
    static IBAN:    OnceLock<Regex> = OnceLock::new();
    static DATE:    OnceLock<Regex> = OnceLock::new();
    static HASHTAG: OnceLock<Regex> = OnceLock::new();
    static MENTION: OnceLock<Regex> = OnceLock::new();
    // Fast path: if we only need >=1 match, use short-circuiting operations.
    let need_one = min_count <= 1;
    match kind {
        EntityKind::Email    => {
            let re = EMAIL.get_or_init(|| Regex::new(r"(?i)\b[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}\b").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::Phone    => {
            let re = PHONE.get_or_init(|| Regex::new(r"\b(?:\+?\d{1,3}[\s\-.]?)?(?:\(?\d{2,4}\)?[\s\-.]?){2,4}\d{2,4}\b").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::Url      => {
            let re = URL.get_or_init(|| Regex::new(r"(?i)\bhttps?://[a-z0-9.\-]+(?:/[^\s]*)?").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::Currency => {
            let re = CURR.get_or_init(|| Regex::new(r"(?:[\$£€¥]\s?\d{1,3}(?:[,.]\d{3})*(?:\.\d+)?|\b\d+(?:[.,]\d+)?\s?(?:USD|EUR|GBP|JPY)\b)").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::IpAddress => {
            let v4 = IP.get_or_init(|| Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|1?\d{1,2})\.){3}(?:25[0-5]|2[0-4]\d|1?\d{1,2})\b").unwrap());
            if need_one {
                return (v4.is_match(input) || input.split_whitespace().any(|tok| looks_like_ipv6(tok.trim_matches(|c: char| !c.is_alphanumeric() && c != ':')))) as usize;
            }
            let mut count = regex_count_early(v4, input, min_count);
            if count >= min_count as usize { return count; }
            for tok in input.split_whitespace() {
                if looks_like_ipv6(tok.trim_matches(|c: char| !c.is_alphanumeric() && c != ':')) {
                    count += 1;
                    if count >= min_count as usize { break; }
                }
            }
            count
        }
        EntityKind::CreditCard => {
            let re = CARD.get_or_init(|| Regex::new(r"\b(?:\d[ -]?){13,19}\b").unwrap());
            if need_one { return re.find(input).map_or(0, |m| luhn_check(m.as_str()) as usize); }
            let mut count = 0usize;
            for m in re.find_iter(input) {
                if luhn_check(m.as_str()) {
                    count += 1;
                    if count >= min_count as usize { break; }
                }
            }
            count
        }
        EntityKind::Iban => {
            let re = IBAN.get_or_init(|| Regex::new(r"(?i)\b[A-Z]{2}\d{2}[A-Z0-9]{10,30}\b").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::DateIso  => {
            let re = DATE.get_or_init(|| Regex::new(r"\b\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01])\b").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::Hashtag  => {
            let re = HASHTAG.get_or_init(|| Regex::new(r"(?:^|[\s(\[{,;:])#[A-Za-z][\w]{0,49}").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::Mention  => {
            let re = MENTION.get_or_init(|| Regex::new(r"(?:^|[\s(\[{,;:])@[A-Za-z0-9_][\w.\-]{0,49}").unwrap());
            if need_one { return re.is_match(input) as usize; }
            regex_count_early(re, input, min_count)
        }
        EntityKind::Emoji    => {
            if need_one { return input.chars().any(is_emoji_char) as usize; }
            let mut count = 0usize;
            for c in input.chars() {
                if is_emoji_char(c) {
                    count += 1;
                    if count >= min_count as usize { break; }
                }
            }
            count
        }
    }
}

#[inline]
fn word_contains(hay: &str, needle: &str) -> bool {
    if needle.is_empty() { return true; }
    let nbytes = needle.as_bytes();
    let hbytes = hay.as_bytes();
    if nbytes.len() > hbytes.len() { return false; }
    let last = hbytes.len() - nbytes.len();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    'outer: for i in 0..=last {
        if !hay.is_char_boundary(i) || !hay.is_char_boundary(i + nbytes.len()) { continue; }
        if &hbytes[i..i + nbytes.len()] != nbytes { continue; }
        // Boundary check on the byte before and after.
        if i > 0 {
            let prev = hbytes[i - 1];
            if is_word(prev) { continue 'outer; }
            // Multi-byte char before? char_boundary at i guarantees this prev is a complete byte; if it's part of a non-ASCII char, treat as word-char.
            if prev >= 0x80 { continue 'outer; }
        }
        let after_idx = i + nbytes.len();
        if after_idx < hbytes.len() {
            let next = hbytes[after_idx];
            if is_word(next) { continue 'outer; }
            if next >= 0x80 { continue 'outer; }
        }
        return true;
    }
    false
}

#[inline]
fn sentence_count(s: &str) -> usize {
    s.split(|c: char| matches!(c, '.' | '!' | '?'))
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .count()
}

/// Zero-width and BOM-style invisible characters that appear in homoglyph/
/// phishing payloads. Whitespace ' ' / '\n' / '\t' are considered visible.
#[inline]
fn is_invisible_char(c: char) -> bool {
    matches!(c,
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{200E}' | '\u{200F}' |
        '\u{202A}'..='\u{202E}' |
        '\u{2060}'..='\u{2064}' |
        '\u{FEFF}' |
        '\u{180E}' | '\u{00AD}'
    )
}

/// Classify a character into one of the supported script families.
/// Punctuation, digits, and ASCII whitespace return `None`.
fn classify_script(c: char) -> Option<&'static str> {
    if c.is_ascii_alphabetic() { return Some("latin"); }
    let cp = c as u32;
    match cp {
        0x00C0..=0x024F | 0x1E00..=0x1EFF              => Some("latin"),
        0x0370..=0x03FF | 0x1F00..=0x1FFF              => Some("greek"),
        0x0400..=0x04FF | 0x0500..=0x052F              => Some("cyrillic"),
        0x0590..=0x05FF                                => Some("hebrew"),
        0x0600..=0x06FF | 0x0750..=0x077F              => Some("arabic"),
        0x0900..=0x097F                                => Some("devanagari"),
        0x0E00..=0x0E7F                                => Some("thai"),
        0x3040..=0x309F                                => Some("hiragana"),
        0x30A0..=0x30FF                                => Some("katakana"),
        0xAC00..=0xD7AF | 0x1100..=0x11FF              => Some("hangul"),
        0x4E00..=0x9FFF | 0x3400..=0x4DBF              => Some("han"),
        _ => None,
    }
}

#[inline]
fn char_in_script(c: char, script: &str) -> bool {
    matches!(classify_script(c), Some(s) if s == script)
}

fn token_uses_multiple_scripts(tok: &str) -> bool {
    let mut seen: Option<&'static str> = None;
    for c in tok.chars() {
        if let Some(s) = classify_script(c) {
            match seen {
                None => seen = Some(s),
                Some(prev) if prev != s => return true,
                _ => {}
            }
        }
    }
    false
}

/// Validate a putative credit-card number using the Luhn checksum.
/// `s` may contain spaces or hyphens between digit groups.
fn luhn_check(s: &str) -> bool {
    let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&digits.len()) { return false; }
    let mut sum = 0u32;
    let mut alt = false;
    for &d in digits.iter().rev() {
        let mut x = d;
        if alt { x *= 2; if x > 9 { x -= 9; } }
        sum += x;
        alt = !alt;
    }
    sum % 10 == 0
}

/// Conservative IPv6 sniffer: token contains at least 2 colons, uses only
/// hex digits + colons, has at least two non-empty hex groups. Avoids the
/// regex-of-doom by leaning on string scans.
fn looks_like_ipv6(tok: &str) -> bool {
    if tok.matches(':').count() < 2 { return false; }
    if !tok.chars().all(|c| c.is_ascii_hexdigit() || c == ':') { return false; }
    let groups: Vec<&str> = tok.split(':').filter(|g| !g.is_empty()).collect();
    if groups.len() < 2 { return false; }
    groups.iter().all(|g| g.len() <= 4)
}

/// Fast path for the common emoji ranges. Not perfect; covers the ranges
/// most often used in real-world inputs (basic emoticons, transport, symbols,
/// supplemental, flags, modifiers).
#[inline]
fn is_emoji_char(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x1F300..=0x1F5FF | // misc symbols & pictographs
        0x1F600..=0x1F64F | // emoticons
        0x1F680..=0x1F6FF | // transport & map
        0x1F700..=0x1F77F |
        0x1F780..=0x1F7FF |
        0x1F800..=0x1F8FF |
        0x1F900..=0x1F9FF | // supplemental
        0x1FA00..=0x1FA6F |
        0x1FA70..=0x1FAFF |
        0x2600..=0x26FF   | // misc symbols
        0x2700..=0x27BF   | // dingbats
        0x1F1E6..=0x1F1FF   // regional indicators (flags)
    )
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

    // ---------- v3 predicate tests ----------

    #[test]
    fn word_contains_any_respects_word_boundary() {
        let yaml = r#"
name: w
rules:
  - id: cat
    when: { word_contains_any: ["cat"] }
    classify: hit
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "the cat sat").classification, "hit");
        // "category" must NOT match — substring "cat" is inside a longer word.
        assert_eq!(evaluate(&p, "the category list").classification, "ok");
    }

    #[test]
    fn starts_and_ends_with_any() {
        let yaml = r#"
name: bookends
rules:
  - id: salutation
    when: { starts_with_any: ["dear ", "hello "] }
    classify: greeted
    confidence: 0.9
  - id: signoff
    when: { ends_with_any: ["regards", "thanks", "cheers"] }
    classify: signed
    confidence: 0.8
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "Dear Alice, please...").classification, "greeted");
        assert_eq!(evaluate(&p, "...Best regards").classification, "signed");
        assert_eq!(evaluate(&p, "no greeting or signoff body").classification, "ok");
    }

    #[test]
    fn sentence_and_char_and_line_counts() {
        let yaml = r#"
name: shape
rules:
  - id: too_long
    when: { sentences: { min: 4 } }
    classify: long
    confidence: 0.9
  - id: too_wide
    when: { chars: { min: 5000 } }
    classify: wide
    confidence: 0.95
  - id: too_tall
    when: { lines: { min: 50 } }
    classify: tall
    confidence: 0.7
"#;
        let p = compile_yaml(yaml).unwrap();
        let four = "One. Two! Three? Four.";
        assert_eq!(evaluate(&p, four).classification, "long");
        let big = "x".repeat(5001);
        assert_eq!(evaluate(&p, &big).classification, "wide");
        let tall = "x\n".repeat(60);
        assert_eq!(evaluate(&p, &tall).classification, "tall");
        assert_eq!(evaluate(&p, "Just one short sentence.").classification, "ok");
    }

    #[test]
    fn digit_and_punctuation_ratio() {
        let yaml = r#"
name: ratios
rules:
  - id: numeric_dump
    when: { digit_ratio_above: { min_ratio: 0.5 } }
    classify: digits
    confidence: 0.9
  - id: punct_burst
    when: { punctuation_ratio_above: { min_ratio: 0.4 } }
    classify: punct
    confidence: 0.8
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "12345 67890 ab").classification, "digits");
        assert_eq!(evaluate(&p, "!!!??? ... !!!").classification, "punct");
        assert_eq!(evaluate(&p, "ordinary prose with words").classification, "ok");
    }

    #[test]
    fn repeated_char_run_catches_yelling_and_stuttering() {
        let yaml = r#"
name: rep
rules:
  - id: spammy
    when: { repeated_char_run: { min_run: 5 } }
    classify: spam
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "soooooo cool").classification, "spam");
        assert_eq!(evaluate(&p, "what?!!!!!").classification, "spam");
        assert_eq!(evaluate(&p, "perfectly normal text").classification, "ok");
    }

    #[test]
    fn repeated_token_detects_copy_paste() {
        let yaml = r#"
name: copy
rules:
  - id: spam
    when: { repeated_token: { min_count: 4 } }
    classify: spam
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(
            evaluate(&p, "buy buy buy buy this product now").classification,
            "spam"
        );
        assert_eq!(evaluate(&p, "buy our product today").classification, "ok");
    }

    #[test]
    fn type_token_ratio_below_flags_low_diversity() {
        let yaml = r#"
name: ttr
rules:
  - id: low_diversity
    when: { type_token_ratio_below: { max_ratio: 0.4 } }
    classify: low_div
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(
            evaluate(&p, "buy buy buy buy buy buy now now now now").classification,
            "low_div"
        );
        assert_eq!(
            evaluate(&p, "every word in this sentence is distinct lexically").classification,
            "ok"
        );
    }

    #[test]
    fn invisible_char_and_mixed_script_detection() {
        let yaml = r#"
name: phish
rules:
  - id: zwsp
    when: { has_invisible_chars: true }
    classify: hidden
    confidence: 0.95
  - id: mixed
    when: { has_mixed_script_token: true }
    classify: homoglyph
    confidence: 0.95
"#;
        let p = compile_yaml(yaml).unwrap();
        let zwsp = "Pay\u{200B}Pal account update";
        let mixed = "Login to Pаypal now"; // Cyrillic 'а' inside Latin
        let r1 = evaluate(&p, zwsp);
        assert!(r1.triggered.iter().any(|t| t.id == "zwsp"));
        let r2 = evaluate(&p, mixed);
        assert!(r2.triggered.iter().any(|t| t.id == "mixed"));
        assert_eq!(evaluate(&p, "Login to Paypal now").classification, "ok");
    }

    #[test]
    fn script_is_filter() {
        let yaml = r#"
name: scripts
rules:
  - id: cyr
    when: { script_is: ["cyrillic"] }
    classify: cyrillic
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(evaluate(&p, "Привет, мир!").classification, "cyrillic");
        assert_eq!(evaluate(&p, "Hello world").classification, "ok");
    }

    #[test]
    fn new_entity_kinds_recognized() {
        let yaml = r#"
name: pii
rules:
  - id: card
    when: { has_entity: { kind: credit_card } }
    classify: card
    confidence: 0.95
  - id: ip
    when: { has_entity: { kind: ip_address } }
    classify: ip
    confidence: 0.9
  - id: iban_hit
    when: { has_entity: { kind: iban } }
    classify: iban
    confidence: 0.9
  - id: dated
    when: { has_entity: { kind: date_iso } }
    classify: dated
    confidence: 0.7
  - id: hash
    when: { has_entity: { kind: hashtag } }
    classify: hashtagged
    confidence: 0.6
  - id: ment
    when: { has_entity: { kind: mention } }
    classify: mentioned
    confidence: 0.6
  - id: emo
    when: { has_entity: { kind: emoji } }
    classify: emoji
    confidence: 0.5
"#;
        let p = compile_yaml(yaml).unwrap();
        // 4242 4242 4242 4242 is a Visa Luhn-valid test card.
        assert!(evaluate(&p, "card 4242 4242 4242 4242").triggered.iter().any(|t| t.id == "card"));
        assert!(evaluate(&p, "server at 192.168.1.42 down").triggered.iter().any(|t| t.id == "ip"));
        assert!(evaluate(&p, "wire to DE89370400440532013000 today").triggered.iter().any(|t| t.id == "iban_hit"));
        assert!(evaluate(&p, "due 2026-05-30").triggered.iter().any(|t| t.id == "dated"));
        assert!(evaluate(&p, "look at #urgent now").triggered.iter().any(|t| t.id == "hash"));
        assert!(evaluate(&p, "hi @alice please review").triggered.iter().any(|t| t.id == "ment"));
        assert!(evaluate(&p, "nice job 🎉🎉").triggered.iter().any(|t| t.id == "emo"));
    }

    #[test]
    fn detects_catalan_via_predicate() {
        let yaml = r#"
name: lang
rules:
  - id: is_catalan
    when: { language_is: ["ca"] }
    classify: ca
    confidence: 0.9
"#;
        let p = compile_yaml(yaml).unwrap();
        assert_eq!(
            evaluate(&p, "Bon dia! Aquesta és la nostra política d'ús.").classification,
            "ca"
        );
    }

    #[test]
    fn catalan_semantic_match_works() {
        // The Catalan synonym dict maps "finalitzar"/"cancellar" → "acabar" and
        // "acord"/"conveni" → "contracte", so synonym variants of the example
        // sentence should land on the same hashed buckets after stemming.
        let yaml = r#"
name: ca_intent
rules:
  - id: cancel_intent
    when:
      semantic_match:
        examples: ["l'usuari vol cancellar el contracte"]
        threshold: 0.3
        language: "ca"
    classify: cancel
    confidence: 0.9
default: { classify: ok, confidence: 1.0 }
"#;
        let p = compile_yaml(yaml).unwrap();
        // Variant phrasing using "acabeu" + "acord" — both expected to canonicalize
        // to "acabar" / "contracte".
        assert_eq!(
            evaluate(&p, "acabeu el meu acord").classification,
            "cancel"
        );
        assert_eq!(
            evaluate(&p, "tinc fam i pluja").classification,
            "ok"
        );
    }
}
