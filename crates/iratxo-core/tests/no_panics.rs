//! Property tests: deterministic, no-panic guarantees.
//!
//! These iterations use a small deterministic LCG instead of a heavy fuzz
//! framework, so they ship as plain `cargo test`. The fuzz crate at
//! `fuzz/` covers the same surface for users who want extended runs.

use iratxo_core::{compile_yaml, decode, evaluate};

fn lcg(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 32) as u32
}

fn rand_bytes(seed: &mut u64, n: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n { v.push(lcg(seed) as u8); }
    v
}

#[test]
fn decoder_never_panics_on_garbage() {
    // Throw 10K random byte strings of varying length at decode().
    // Every one must produce Ok or a typed Err, never a panic/abort.
    let mut seed = 0xc0ffeeu64;
    for _ in 0..10_000 {
        let len = (lcg(&mut seed) % 200) as usize;
        let bytes = rand_bytes(&mut seed, len);
        let _ = decode(&bytes);
    }
}

#[test]
fn evaluate_never_panics_on_random_input() {
    let yaml = r#"
name: random_input
rules:
  - id: r1
    when: { contains_any: ["foo", "bar"] }
    classify: a
    confidence: 0.5
  - id: r2
    when:
      regex: "[a-z]+"
    classify: b
    confidence: 0.5
  - id: r3
    when: { semantic_match: { examples: ["hello world"], threshold: 0.3 } }
    classify: c
    confidence: 0.5
"#;
    let program = compile_yaml(yaml).unwrap();

    let mut seed = 0xfeedfacefeedfaceu64;
    for _ in 0..2_000 {
        let len = (lcg(&mut seed) % 1024) as usize;
        let bytes = rand_bytes(&mut seed, len);
        let s = String::from_utf8_lossy(&bytes);
        let _ = evaluate(&program, &s);
    }
}

#[test]
fn evaluate_is_deterministic() {
    let yaml = r#"
name: det
rules:
  - id: r
    when: { semantic_match: { examples: ["please cancel my contract"], threshold: 0.3, language: "en" } }
    classify: cancel
    confidence: 0.9
"#;
    let program = compile_yaml(yaml).unwrap();
    let input = "Hi support, I need to terminate my agreement";

    let r1 = evaluate(&program, input);
    let r2 = evaluate(&program, input);
    let r3 = evaluate(&program, input);

    assert_eq!(r1.classification, r2.classification);
    assert_eq!(r2.classification, r3.classification);
    assert_eq!(r1.confidence, r2.confidence);
    assert_eq!(r2.confidence, r3.confidence);
}
