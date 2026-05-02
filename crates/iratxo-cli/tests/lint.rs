//! Tests for `iratxo lint` covering the verbose success summary and the
//! detailed failure output for parse errors and content-lint validations.

use std::path::PathBuf;
use std::process::{Command, Output};

fn iratxo_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_iratxo"))
}

fn run_lint(yaml: &str) -> Output {
    let dir = tempfile::tempdir().unwrap();
    let yaml_path = dir.path().join("rule.yaml");
    std::fs::write(&yaml_path, yaml).unwrap();
    Command::new(iratxo_bin())
        .args(["lint", yaml_path.to_str().unwrap()])
        .output()
        .unwrap()
}

fn assert_lint_fails_with(yaml: &str, expected_substrings: &[&str]) {
    let out = run_lint(yaml);
    assert!(!out.status.success(), "lint should have failed; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in expected_substrings {
        assert!(stderr.contains(needle), "stderr missing {:?}\n--- stderr ---\n{stderr}", needle);
    }
}

const GOOD_YAML: &str = r#"name: compliance
description: Sample compliance rules.
rules:
  - id: forbidden_phrase
    when:
      contains_any: ["guaranteed refund"]
    classify: review_required
    confidence: 0.95
    explanation: "Refund language detected."
  - id: medical_claim
    when:
      regex: "\\b(FDA approved)\\b"
    classify: blocked
    confidence: 0.99
default:
  classify: ok
  confidence: 1.0
"#;

#[test]
fn lint_success_prints_verbose_summary() {
    let out = run_lint(GOOD_YAML);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ok: compliance (2 rules)"), "got: {stdout}");
    assert!(stdout.contains("description: Sample compliance rules."), "got: {stdout}");
    // Per-rule summary: id, predicate kind, classification, confidence, explanation.
    assert!(stdout.contains("forbidden_phrase"), "got: {stdout}");
    assert!(stdout.contains("contains_any(1)"), "got: {stdout}");
    assert!(stdout.contains("review_required"), "got: {stdout}");
    assert!(stdout.contains("0.95"), "got: {stdout}");
    assert!(stdout.contains("\"Refund language detected.\""), "got: {stdout}");
    assert!(stdout.contains("medical_claim"), "got: {stdout}");
    assert!(stdout.contains("regex"), "got: {stdout}");
    assert!(stdout.contains("default: ok (conf 1)"), "got: {stdout}");
}

#[test]
fn lint_singular_rule_count() {
    let yaml = r#"name: solo
rules:
  - id: only
    when: { contains_any: ["x"] }
    classify: hit
"#;
    let out = run_lint(yaml);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ok: solo (1 rule)"), "got: {stdout}");
}

#[test]
fn lint_parse_error_shows_file_path_line_and_snippet() {
    // Unbalanced bracket — serde_yaml reports a location.
    let yaml = "name: t\nrules: [\n  - id: a\n";
    let out = run_lint(yaml);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error: lint failed for"), "got: {stderr}");
    assert!(stderr.contains("rule.yaml"), "got: {stderr}");
    assert!(stderr.contains("parse error"), "got: {stderr}");
}

#[test]
fn lint_duplicate_rule_id_fails() {
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: dup
    when: { contains_any: ["x"] }
    classify: a
  - id: dup
    when: { contains_any: ["y"] }
    classify: b
"#,
        &["duplicate rule id", "dup"],
    );
}

#[test]
fn lint_then_to_unknown_rule_fails() {
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
    then: ["ghost"]
"#,
        &["chains to unknown rule", "ghost"],
    );
}

#[test]
fn lint_then_self_chain_fails() {
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
    then: ["a"]
"#,
        &["chains to itself"],
    );
}

#[test]
fn lint_invalid_regex_fails_with_pattern_and_rule_id() {
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: bad_pattern
    when: { regex: "(unclosed" }
    classify: hit
"#,
        &["regex pattern", "(unclosed", "bad_pattern"],
    );
}

#[test]
fn lint_rule_confidence_out_of_range_fails() {
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
    confidence: 1.7
"#,
        &["confidence 1.7", "out of [0,1]"],
    );
}

#[test]
fn lint_default_confidence_out_of_range_fails() {
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
default:
  classify: ok
  confidence: -0.5
"#,
        &["default confidence", "out of [0,1]"],
    );
}

#[test]
fn lint_unknown_entity_kind_threads_rule_id() {
    // Validation errors raised inside lower_predicate should be prefixed
    // with the rule id by lower_rule.
    assert_lint_fails_with(
        r#"name: t
rules:
  - id: pii_check
    when:
      has_entity:
        kind: "ssn"
    classify: hit
"#,
        &["pii_check", "unknown entity kind", "ssn"],
    );
}

#[test]
fn lint_missing_file_reports_clearly() {
    // No tempfile — pass a nonexistent path directly.
    let out = Command::new(iratxo_bin())
        .args(["lint", "/no/such/iratxo/rule.yaml"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot stat"), "got: {stderr}");
    assert!(stderr.contains("/no/such/iratxo/rule.yaml"), "got: {stderr}");
    // The trailing summary lists the failed path.
    assert!(stderr.contains("lint failed:"), "got: {stderr}");
}

#[test]
fn lint_multiple_files_lists_failed_paths_in_summary() {
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("good.yaml");
    let bad1 = dir.path().join("bad-regex.yaml");
    let bad2 = dir.path().join("dup.yaml");
    std::fs::write(&good, GOOD_YAML).unwrap();
    std::fs::write(&bad1, r#"name: t
rules:
  - id: bad
    when: { regex: "(unclosed" }
    classify: hit
"#).unwrap();
    std::fs::write(&bad2, r#"name: t
rules:
  - id: dup
    when: { contains_any: ["x"] }
    classify: a
  - id: dup
    when: { contains_any: ["y"] }
    classify: b
"#).unwrap();

    let out = Command::new(iratxo_bin())
        .args(["lint", good.to_str().unwrap(), bad1.to_str().unwrap(), bad2.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Per-file headers in stdout.
    assert!(stdout.contains("== "), "got stdout: {stdout}");
    assert!(stdout.contains("good.yaml"), "got stdout: {stdout}");
    // Trailing summary in stderr lists ONLY the failed files.
    assert!(stderr.contains("lint failed: 2 files"), "got stderr: {stderr}");
    assert!(stderr.contains("bad-regex.yaml"), "got stderr: {stderr}");
    assert!(stderr.contains("dup.yaml"), "got stderr: {stderr}");
    assert!(!stderr.contains("good.yaml"),
        "summary should not list passing files; got stderr: {stderr}");
}

#[test]
fn lint_all_pass_summary_prints_count() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.yaml");
    let b = dir.path().join("b.yml");
    std::fs::write(&a, GOOD_YAML).unwrap();
    std::fs::write(&b, r#"name: u
rules:
  - id: only
    when: { contains_any: ["x"] }
    classify: hit
"#).unwrap();

    let out = Command::new(iratxo_bin())
        .args(["lint", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("lint ok: 2 files passed"), "got: {stdout}");
}

#[test]
fn lint_directory_recurses_and_picks_up_yaml_yml() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested/deep");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(dir.path().join("top.yaml"), GOOD_YAML).unwrap();
    std::fs::write(nested.join("inner.yml"), r#"name: inner
rules:
  - id: a
    when: { contains_any: ["x"] }
    classify: hit
"#).unwrap();
    // Non-yaml files in the tree must be ignored.
    std::fs::write(dir.path().join("README.md"), "not yaml").unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not yaml").unwrap();

    let out = Command::new(iratxo_bin())
        .args(["lint", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("top.yaml"), "got: {stdout}");
    assert!(stdout.contains("inner.yml"), "got: {stdout}");
    assert!(stdout.contains("lint ok: 2 files passed"), "got: {stdout}");
}

#[test]
fn lint_directory_with_one_bad_file_lists_only_that_path() {
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("ok.yaml");
    let bad = dir.path().join("broken.yaml");
    std::fs::write(&good, GOOD_YAML).unwrap();
    std::fs::write(&bad, r#"name: t
rules:
  - id: a
    when: { regex: "(unclosed" }
    classify: hit
"#).unwrap();

    let out = Command::new(iratxo_bin())
        .args(["lint", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("lint failed: 1 file"), "got: {stderr}");
    assert!(stderr.contains("broken.yaml"), "got: {stderr}");
    assert!(!stderr.contains("ok.yaml: "), "summary should not list passing files; got: {stderr}");
}

#[test]
fn lint_empty_directory_warns_but_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(iratxo_bin())
        .args(["lint", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    // Nothing failed, so exit 0.
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no .yaml/.yml files"), "got: {stderr}");
}

#[test]
fn lint_no_args_errors_via_clap() {
    let out = Command::new(iratxo_bin()).args(["lint"]).output().unwrap();
    assert!(!out.status.success());
    // clap's "required arg missing" message
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.to_lowercase().contains("required") || stderr.contains("usage"),
        "got: {stderr}");
}
