//! End-to-end test: compile YAML → IR, run IR through the engine.wasm via the
//! `iratxo` CLI, assert the JSON verdict.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf()
}

fn iratxo_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_iratxo"))
}

fn engine_wasm() -> PathBuf {
    workspace_root()
        .join("crates/iratxo-engine/target/wasm32-unknown-unknown/release/iratxo_engine.wasm")
}

fn run_pipeline(yaml: &str, input: &str) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let yaml_path = dir.path().join("rule.yaml");
    let ir_path = dir.path().join("rule.iratxo");
    let input_path = dir.path().join("input.txt");
    std::fs::write(&yaml_path, yaml).unwrap();
    std::fs::write(&input_path, input).unwrap();

    let out = Command::new(iratxo_bin())
        .args(["build", yaml_path.to_str().unwrap(), "-o", ir_path.to_str().unwrap()])
        .output().unwrap();
    assert!(out.status.success(), "build failed: {}", String::from_utf8_lossy(&out.stderr));

    let out = Command::new(iratxo_bin())
        .args(["run", ir_path.to_str().unwrap(), input_path.to_str().unwrap(),
               "--engine", engine_wasm().to_str().unwrap()])
        .output().unwrap();
    assert!(out.status.success(), "run failed: {}", String::from_utf8_lossy(&out.stderr));

    let stdout = String::from_utf8(out.stdout).unwrap();
    let last_line = stdout.lines().last().unwrap();
    serde_json::from_str(last_line).unwrap()
}

const COMPLIANCE_YAML: &str = r#"
name: outbound_compliance
rules:
  - id: no_refund_guarantee
    when:
      contains_any: ["guaranteed refund", "100% refund"]
    classify: review_required
    confidence: 0.95
    explanation: "Forbidden refund guarantee"
  - id: prohibited_claim
    when:
      regex: "\\b(cure|FDA approved)\\b"
    classify: blocked
    confidence: 0.99
default:
  classify: ok
  confidence: 1.0
"#;

#[test]
fn clean_input_classifies_ok() {
    if !engine_wasm().exists() {
        eprintln!("skipping: engine.wasm not built");
        return;
    }
    let v = run_pipeline(COMPLIANCE_YAML, "Hi there, this is a normal email.");
    assert_eq!(v["classification"], "ok");
    assert_eq!(v["triggered"].as_array().unwrap().len(), 0);
}

#[test]
fn forbidden_phrase_triggers_review() {
    if !engine_wasm().exists() { return; }
    let v = run_pipeline(COMPLIANCE_YAML, "Sign up for a guaranteed refund today");
    assert_eq!(v["classification"], "review_required");
    let ids: Vec<&str> = v["triggered"].as_array().unwrap().iter()
        .map(|t| t["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"no_refund_guarantee"));
}

#[test]
fn semantic_match_through_wasm() {
    if !engine_wasm().exists() { return; }
    let yaml = r#"
name: cancel_detector
rules:
  - id: cancel_intent
    when:
      semantic_match:
        examples: ["the user wants to cancel their agreement"]
        threshold: 0.4
    classify: cancellation_intent
    confidence: 0.9
"#;
    let v = run_pipeline(yaml, "Please terminate my contract immediately");
    assert_eq!(v["classification"], "cancellation_intent");

    let v2 = run_pipeline(yaml, "I love your product, the colors are great");
    assert_eq!(v2["classification"], "ok");
}

#[test]
fn semantic_match_spanish_through_wasm() {
    if !engine_wasm().exists() { return; }
    // The built-in synonym dict is English-only; rules in other languages
    // supply their own canonical→synonyms map via `synonyms`.
    let yaml = r#"
name: cancelar_es
rules:
  - id: intencion_cancelar
    when:
      semantic_match:
        examples: ["el usuario quiere cancelar su contrato"]
        threshold: 0.3
        language: "es"
        synonyms:
          cancelar: ["terminar", "rescindir", "anular"]
          contrato: ["acuerdo", "convenio"]
    classify: cancelacion
    confidence: 0.9
"#;
    let v = run_pipeline(yaml, "Por favor, terminen mi contrato cuanto antes");
    assert_eq!(v["classification"], "cancelacion");
}

#[test]
fn semantic_match_basque_through_wasm() {
    if !engine_wasm().exists() { return; }
    let yaml = r#"
name: liburu_eu
rules:
  - id: liburu_aipamen
    when:
      semantic_match:
        examples: ["liburu bat irakurtzen ari naiz"]
        threshold: 0.2
        language: "eu"
    classify: liburu_aipamen
    confidence: 0.9
"#;
    let v = run_pipeline(yaml, "Etxean nago liburuak irakurtzen");
    assert_eq!(v["classification"], "liburu_aipamen");
}

#[test]
fn highest_confidence_rule_wins() {
    if !engine_wasm().exists() { return; }
    let v = run_pipeline(COMPLIANCE_YAML, "guaranteed refund and FDA approved");
    assert_eq!(v["classification"], "blocked");
    assert!((v["confidence"].as_f64().unwrap() - 0.99).abs() < 1e-6);
    assert_eq!(v["triggered"].as_array().unwrap().len(), 2);
}
