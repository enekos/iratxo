//! Microbenchmarks: how fast is the interpreter on representative inputs?
//!
//! Three workloads:
//!   - `keyword`     — a 4-rule pack (`contains_any`/regex) over a short input.
//!   - `regex_heavy` — a 6-rule regex pack over a 5KB doc; exercises the
//!                     per-evaluate regex cache.
//!   - `semantic`    — a 1-rule `semantic_match` over a paragraph; exercises
//!                     tokenizer + stemmer + hashing-trick path.
//!
//! Run: `cargo bench -p iratxo-core`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iratxo_core::{compile_yaml, evaluate};

const KEYWORD_YAML: &str = r#"
name: keyword_pack
rules:
  - id: refund
    when: { contains_any: ["guaranteed refund", "money back"] }
    classify: review
    confidence: 0.9
  - id: phone
    when:
      regex: "\\b\\d{3}-\\d{4}\\b"
      case_sensitive: true
    classify: pii
    confidence: 0.8
  - id: disclaimer
    when: { not_contains_any: ["This is not financial advice"] }
    classify: review
    confidence: 0.5
  - id: short
    when: { max_length: 5 }
    classify: too_short
    confidence: 0.3
"#;

const REGEX_YAML: &str = r#"
name: regex_pack
rules:
  - id: r1
    when: { regex: "(?i)cancel" }
    classify: a
    confidence: 0.9
  - id: r2
    when: { regex: "(?i)terminate" }
    classify: b
    confidence: 0.9
  - id: r3
    when: { regex: "(?i)refund" }
    classify: c
    confidence: 0.9
  - id: r4
    when: { regex: "\\b[A-Z]{2,}\\b" }
    classify: d
    confidence: 0.5
  - id: r5
    when: { regex: "\\d{4}" }
    classify: e
    confidence: 0.5
  - id: r6
    when: { regex: "https?://\\S+" }
    classify: f
    confidence: 0.5
"#;

const SEMANTIC_YAML: &str = r#"
name: semantic_pack
rules:
  - id: cancel_intent
    when:
      semantic_match:
        examples: ["the user wants to cancel their agreement"]
        threshold: 0.3
        language: "en"
    classify: cancellation
    confidence: 0.9
"#;

const KEYWORD_INPUT: &str =
    "Hi support, please send me a guaranteed refund. My number is 555-1234. \
     This is not financial advice.";

const SEMANTIC_INPUT: &str =
    "Hi support, I'd like to terminate my contract with you. \
     This is not financial advice, just a personal decision.";

fn long_input() -> String {
    let para = "Some boilerplate text about contracts and policies. ";
    let mut s = String::with_capacity(5_000);
    while s.len() < 5_000 {
        s.push_str(para);
    }
    s.push_str("Please cancel my subscription. Visit https://example.com for FAQ.");
    s
}

fn bench_keyword(c: &mut Criterion) {
    let p = compile_yaml(KEYWORD_YAML).unwrap();
    c.bench_function("keyword/short", |b| {
        b.iter(|| evaluate(black_box(&p), black_box(KEYWORD_INPUT)))
    });
}

fn bench_regex_heavy(c: &mut Criterion) {
    let p = compile_yaml(REGEX_YAML).unwrap();
    let input = long_input();
    c.bench_function("regex/5kb_doc_6_rules", |b| {
        b.iter(|| evaluate(black_box(&p), black_box(&input)))
    });
}

fn bench_semantic(c: &mut Criterion) {
    let p = compile_yaml(SEMANTIC_YAML).unwrap();
    c.bench_function("semantic/paragraph", |b| {
        b.iter(|| evaluate(black_box(&p), black_box(SEMANTIC_INPUT)))
    });
}

criterion_group!(benches, bench_keyword, bench_regex_heavy, bench_semantic);
criterion_main!(benches);
