//! aatxe-driven bench runner for iratxo-core.
//!
//! Mirrors the three workloads in `crates/iratxo-core/benches/engine.rs`
//! (criterion) but emits a single aatxe `RunReport` JSON on stdout, suitable
//! for `aatxe run --lang rust --runner "cargo run -p iratxo-bench-runner \
//! --release -q --bin iratxo-aatxe-runner --"`.
//!
//! WIP: bench surface intentionally identical to the criterion file so the two
//! harnesses can be compared 1:1 before retiring criterion.

use aatxe_bench::{bench, keep, Suite};
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

fn main() {
    let mut suite = Suite::new("iratxo-core");

    let keyword_program = compile_yaml(KEYWORD_YAML).expect("keyword pack compiles");
    bench(&mut suite, "keyword/short", || {
        keep(evaluate(&keyword_program, KEYWORD_INPUT));
    });

    let regex_program = compile_yaml(REGEX_YAML).expect("regex pack compiles");
    let regex_input = long_input();
    bench(&mut suite, "regex/5kb_doc_6_rules", || {
        keep(evaluate(&regex_program, &regex_input));
    });

    let semantic_program = compile_yaml(SEMANTIC_YAML).expect("semantic pack compiles");
    bench(&mut suite, "semantic/paragraph", || {
        keep(evaluate(&semantic_program, SEMANTIC_INPUT));
    });

    suite.emit_stdout();
}
