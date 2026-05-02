//! Stress-test benchmark with many complex, deeply-nested rules and rich metrics.
//!
//! Workloads:
//!   - `complex/mixed_30_rules` — 30-rule pack with nested ALL/ANY/NOT,
//!     entity detection, regex, semantic, structural, chaining, ratios, counts.
//!   - `complex/nested_deep` — 10 rules with deeply nested predicates (5+ levels).
//!   - `complex/large_doc_entities` — 20KB doc scanned for 10 entity kinds + url domains.
//!
//! Metrics emitted as METRIC lines for autoresearch consumption.

use std::time::{Duration, Instant};

const COMPLEX_YAML: &str = r#"
name: complex_stress_pack
rules:
  # --- simple keyword layers ---
  - id: refund
    when: { contains_any: ["guaranteed refund", "money back", "full reimbursement"] }
    classify: refund_intent
    confidence: 0.92
  - id: phone
    when:
      regex: "\\b\\d{3}-\\d{4}\\b"
      case_sensitive: true
    classify: pii_phone
    confidence: 0.85
  - id: not_disclaimer
    when: { not_contains_any: ["This is not financial advice", "no advice given"] }
    classify: missing_disclaimer
    confidence: 0.6
  - id: short_msg
    when: { max_length: 5 }
    classify: too_short
    confidence: 0.3

  # --- entity detection cluster ---
  - id: has_email
    when: { has_entity: { kind: email, min_count: 1 } }
    classify: pii_email
    confidence: 0.9
  - id: has_url
    when: { has_entity: { kind: url, min_count: 1 } }
    classify: contains_link
    confidence: 0.7
  - id: has_ip
    when: { has_entity: { kind: ip_address, min_count: 1 } }
    classify: pii_ip
    confidence: 0.88
  - id: has_card
    when: { has_entity: { kind: credit_card, min_count: 1 } }
    classify: pii_card
    confidence: 0.95
  - id: has_iban
    when: { has_entity: { kind: iban, min_count: 1 } }
    classify: pii_iban
    confidence: 0.93
  - id: has_date
    when: { has_entity: { kind: date_iso, min_count: 1 } }
    classify: dated
    confidence: 0.5
  - id: has_hashtag
    when: { has_entity: { kind: hashtag, min_count: 2 } }
    classify: social
    confidence: 0.55
  - id: has_mention
    when: { has_entity: { kind: mention, min_count: 1 } }
    classify: social_mention
    confidence: 0.55
  - id: has_emoji
    when: { has_entity: { kind: emoji, min_count: 3 } }
    classify: expressive
    confidence: 0.4

  # --- structural / shape ---
  - id: has_termination_section
    when: { has_section: ["Termination", "Cancellation", "End of Contract"] }
    classify: contract_section
    confidence: 0.8
  - id: too_many_paragraphs
    when: { paragraphs: { min: 10 } }
    classify: long_form
    confidence: 0.5
  - id: max_words_ok
    when: { max_words_per_sentence: 25 }
    classify: readable
    confidence: 0.4

  # --- URL domain filtering ---
  - id: external_link
    when:
      not:
        has_url_to_domain:
          domains: ["example.com", "trusted.org", "docs.example.com"]
          allow_subdomains: true
    classify: external_url
    confidence: 0.75

  # --- ratio / entropy heuristics ---
  - id: shouting
    when: { mostly_uppercase: { min_ratio: 0.7 } }
    classify: shout
    confidence: 0.65
  - id: secret_token
    when:
      token_entropy_above:
        min_bits: 4.0
        min_token_len: 16
    classify: possible_secret
    confidence: 0.85
  - id: high_digits
    when: { digit_ratio_above: { min_ratio: 0.4 } }
    classify: numeric_heavy
    confidence: 0.5
  - id: high_punct
    when: { punctuation_ratio_above: { min_ratio: 0.35 } }
    classify: punct_heavy
    confidence: 0.45

  # --- v3 word-level + boundaries ---
  - id: word_cancel
    when: { word_contains_any: ["cancel", "terminate", "rescind"] }
    classify: cancel_word
    confidence: 0.78
  - id: starts_dear
    when: { starts_with_any: ["Dear", "Hello", "Hi"] }
    classify: formal_greeting
    confidence: 0.4
  - id: ends_regards
    when: { ends_with_any: ["regards", "thanks", "cheers"] }
    classify: formal_signoff
    confidence: 0.4

  # --- repetition / spam signals ---
  - id: char_spam
    when: { repeated_char_run: { min_run: 5 } }
    classify: spam_pattern
    confidence: 0.7
  - id: token_spam
    when: { repeated_token: { min_count: 4 } }
    classify: spam_repeat
    confidence: 0.75
  - id: low_diversity
    when: { type_token_ratio_below: { max_ratio: 0.35 } }
    classify: low_lexical_div
    confidence: 0.6
  - id: invisible_chars
    when: { has_invisible_chars: true }
    classify: obfuscated
    confidence: 0.9
  - id: mixed_script
    when: { has_mixed_script_token: true }
    classify: homoglyph_risk
    confidence: 0.85
  - id: cyrillic_text
    when: { script_is: ["cyrillic"] }
    classify: cyrillic_content
    confidence: 0.5

  # --- semantic ---
  - id: cancel_semantic
    when:
      semantic_match:
        examples:
          - "the user wants to cancel their agreement"
          - "please terminate my subscription"
        threshold: 0.3
        language: "en"
    classify: semantic_cancel
    confidence: 0.82

  # --- chaining demo ---
  - id: chain_trigger
    when: { contains_any: ["alpha"] }
    classify: chain_a
    confidence: 0.5
    then: ["chain_next"]
  - id: chain_next
    when: { always: true }
    classify: chain_b
    confidence: 0.91

default:
  classify: ok
  confidence: 1.0
"#;

const NESTED_DEEP_YAML: &str = r#"
name: nested_deep_pack
rules:
  - id: deep_1
    when:
      all:
        - contains_any: ["start"]
        - any:
            - regex: "\\d{4}"
            - all:
                - not_contains_any: ["exclude"]
                - word_contains_any: ["token"]
        - not:
            any:
              - contains_any: ["bad"]
              - has_invisible_chars: true
    classify: deep_match
    confidence: 0.9
  - id: deep_2
    when:
      all:
        - all:
            - all:
                - contains_any: ["layer1"]
                - any:
                    - contains_any: ["layer2a"]
                    - contains_any: ["layer2b"]
            - not:
                not:
                  contains_any: ["double_neg"]
        - ends_with_any: ["end"]
    classify: very_deep
    confidence: 0.85
  - id: deep_3
    when:
      any:
        - all:
            - starts_with_any: ["A"]
            - ends_with_any: ["Z"]
            - chars: { min: 10, max: 1000 }
        - all:
            - digit_ratio_above: { min_ratio: 0.5 }
            - punctuation_ratio_above: { min_ratio: 0.2 }
            - not:
                has_mixed_script_token: true
        - semantic_match:
            examples: ["deep semantic probe"]
            threshold: 0.25
            language: "en"
    classify: multi_branch
    confidence: 0.8
  - id: deep_4
    when:
      all:
        - has_entity: { kind: email, min_count: 1 }
        - has_entity: { kind: url, min_count: 1 }
        - has_url_to_domain:
            domains: ["example.com"]
            allow_subdomains: true
        - paragraphs: { min: 1, max: 50 }
        - sentences: { min: 1, max: 100 }
        - lines: { min: 1, max: 200 }
    classify: structured_pii
    confidence: 0.92
  - id: deep_5
    when:
      any:
        - repeated_char_run: { min_run: 5 }
        - repeated_token: { min_count: 4 }
        - type_token_ratio_below: { max_ratio: 0.3 }
        - token_entropy_above: { min_bits: 4.5, min_token_len: 20 }
        - mostly_uppercase: { min_ratio: 0.8 }
    classify: anomaly
    confidence: 0.75
  - id: deep_6
    when:
      all:
        - not: { has_invisible_chars: true }
        - not: { has_mixed_script_token: true }
        - not: { script_is: ["cyrillic"] }
        - not: { contains_any: ["blocked"] }
        - word_contains_any: ["safe"]
    classify: clean
    confidence: 0.7
  - id: deep_7
    when:
      all:
        - language_is: ["en"]
        - max_words_per_sentence: 30
        - max_length: 500
        - not_contains_any: ["spam", "scam"]
    classify: english_prose
    confidence: 0.6
  - id: deep_8
    when:
      any:
        - has_section: ["Introduction"]
        - has_section: ["Summary"]
        - all:
            - has_entity: { kind: date_iso, min_count: 1 }
            - has_entity: { kind: hashtag, min_count: 1 }
    classify: doc_type
    confidence: 0.55
  - id: deep_9
    when:
      all:
        - contains_any: ["chain_seed"]
        - always: true
    classify: chain_seed
    confidence: 0.4
    then: ["deep_10"]
  - id: deep_10
    when: { always: true }
    classify: chain_tail
    confidence: 0.95

default:
  classify: ok
  confidence: 1.0
"#;

const LARGE_DOC: &str = r#"
Dear Support Team,

Please find below the details of my request. My contact email is alice@example.com
and you can reach me at 555-0199. The server IP is 192.168.1.42 and the backup is
at 10.0.0.1. I used card 4242 4242 4242 4242 for payment and the IBAN is
DE89370400440532013000. The invoice date is 2025-12-31.

# Introduction

This is the introduction section. We will discuss contracts, refunds, and cancellations.
Please visit https://docs.example.com/policy for the full policy.
Also check https://external-evil.com/phish for bad stuff.

## Termination

Either party may cancel or terminate the agreement with 30 days notice.
Money back is guaranteed for the first 14 days. Full reimbursement applies.
This is not financial advice.

# Summary

Key points: guaranteed refund, money back, full reimbursement.
Contact: alice@example.com or bob@trusted.org. Call 555-0199 or 555-0200.
Server: 192.168.1.42, 10.0.0.1, 172.16.0.1.
Payment: 4242 4242 4242 4242 (Visa).
IBAN: DE89370400440532013000.
Date: 2025-12-31.

Social: #urgent #help #refund @alice @support

Emoji: 🎉🎉🎉 for success! 🚀🚀🚀 for launch!

Some random secret token for API: aZ9bX2qW7eR4tY6uI8oP3sD5fG1hJ0kL
Another one: xK7mN2pQ4rS8tU0vW1yZ3aB5cD6eF7gH

Repeating text: buy buy buy buy this product now now now now.
Also: soooooo cool and what?!!!!!

Cyrillic sample: Привет мир! Это тест.
Invisible: Pay​Pal account update.

Paragraph 1. Paragraph 2. Paragraph 3. Paragraph 4. Paragraph 5.
Paragraph 6. Paragraph 7. Paragraph 8. Paragraph 9. Paragraph 10.
"#;

fn build_20kb_doc() -> String {
    let mut s = String::with_capacity(20_000);
    for i in 0..100 {
        s.push_str(&format!(
            "Line {}: Contact {}@example.com or call 555-{:04}. \
             Server {}.{}.{}.{} Card {} {} {} {} IBAN {}. \\n",
            i,
            i,
            i,
            i % 256,
            i % 256,
            i % 256,
            i % 256,
            i,
            i + 1,
            i + 2,
            i + 3,
            i
        ));
    }
    s.push_str(LARGE_DOC);
    s
}

fn bench_workload(name: &str, program: &iratxo_core::Program, input: &str, iterations: u32) {
    // Warmup
    for _ in 0..iterations / 10 {
        let _ = iratxo_core::evaluate(program, input);
    }

    // Single metrics-collecting run for ratios
    let (_, metrics) = iratxo_core::evaluate_with_metrics(program, input);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = iratxo_core::evaluate(program, input);
    }
    let elapsed = start.elapsed();
    let total_us = elapsed.as_micros() as f64;
    let per_iter_us = total_us / iterations as f64;

    println!("METRIC {}_total_µs={}", name, total_us);
    println!("METRIC {}_per_iter_µs={}", name, per_iter_us);

    // --- rich diagnostic metrics ---
    println!("METRIC {}_rule_evals={}", name, metrics.rule_evals);
    println!("METRIC {}_predicate_evals={}", name, metrics.predicate_evals);
    println!("METRIC {}_predicate_true={}", name, metrics.predicate_true);
    println!("METRIC {}_predicate_false={}", name, metrics.predicate_false);
    println!("METRIC {}_regex_cache_hits={}", name, metrics.regex_cache_hits);
    println!("METRIC {}_regex_cache_misses={}", name, metrics.regex_cache_misses);
    println!("METRIC {}_tokenize_calls={}", name, metrics.tokenize_calls);
    println!("METRIC {}_tokens_produced={}", name, metrics.tokens_produced);
    println!("METRIC {}_semantic_similarity_calls={}", name, metrics.semantic_similarity_calls);
    println!("METRIC {}_entity_detection_calls={}", name, metrics.entity_detection_calls);
    println!("METRIC {}_url_extract_calls={}", name, metrics.url_extract_calls);
    println!("METRIC {}_section_scan_calls={}", name, metrics.section_scan_calls);
    println!("METRIC {}_chain_traversals={}", name, metrics.chain_traversals);
    println!("METRIC {}_triggered_rules={}", name, metrics.triggered_rules);
    println!("METRIC {}_all_short_circuits={}", name, metrics.all_short_circuits);
    println!("METRIC {}_any_short_circuits={}", name, metrics.any_short_circuits);
    println!("METRIC {}_not_short_circuits={}", name, metrics.not_short_circuits);
    println!("METRIC {}_max_chain_depth={}", name, metrics.max_chain_depth);
    println!("METRIC {}_language_detect_calls={}", name, metrics.language_detect_calls);
    println!("METRIC {}_lower_allocations={}", name, metrics.lower_allocations);
    println!("METRIC {}_lower_bytes={}", name, metrics.lower_bytes);
}

fn main() {
    let complex = iratxo_core::compile_yaml(COMPLEX_YAML).unwrap();
    let nested = iratxo_core::compile_yaml(NESTED_DEEP_YAML).unwrap();
    let large_doc = build_20kb_doc();

    let iters = 200;

    bench_workload("complex_mixed", &complex, LARGE_DOC, iters);
    bench_workload("nested_deep", &nested, LARGE_DOC, iters);
    bench_workload("large_doc_entities", &complex, &large_doc, iters / 2);
}
