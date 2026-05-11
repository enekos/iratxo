use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use wasmtime::{Engine, Instance, Module, Store};

#[derive(Parser)]
#[command(name = "iratxo", version, about = "Executable language rules as Wasm plugins")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Compile a YAML rule file to a portable .iratxo binary IR.
    Build {
        /// Path to the YAML rule source.
        input: PathBuf,
        /// Path for the compiled IR. Defaults to `<input>.iratxo`.
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Validate one or more YAML rule files without writing output.
    /// Directories are searched recursively for `*.yaml` and `*.yml`.
    /// Exits non-zero if any file fails; lists failing files at the end.
    Lint {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },
    /// Run a compiled IR against an input file using the Wasm engine.
    Run {
        /// Compiled .iratxo IR.
        rule: PathBuf,
        /// Input text file.
        input: PathBuf,
        /// Path to the engine .wasm. Defaults to the workspace-built artifact.
        #[arg(long)]
        engine: Option<PathBuf>,
    },
    /// Run a compiled IR with the in-process native interpreter (no Wasm).
    RunNative {
        rule: PathBuf,
        input: PathBuf,
    },
    /// Run a YAML golden-file test suite against a rule.
    /// Cases are evaluated natively (no Wasm) for speed.
    Test {
        /// Path to the YAML rule source.
        rule: PathBuf,
        /// Path to a YAML cases file. See `examples/cases/*.yaml`.
        cases: PathBuf,
    },
    /// Sign a compiled .iratxo with an Ed25519 keypair, writing a `.iratxo.sig`.
    Sign {
        rule: PathBuf,
        /// Path to a 32-byte Ed25519 secret key (raw, not PEM).
        #[arg(long)]
        key: PathBuf,
    },
    /// Verify a compiled .iratxo against its `.iratxo.sig` using the given pub key.
    Verify {
        rule: PathBuf,
        #[arg(long)]
        pubkey: PathBuf,
    },
    /// Generate a fresh Ed25519 keypair to disk.
    Keygen {
        /// Where to write the secret key (32 raw bytes).
        #[arg(long)]
        secret: PathBuf,
        /// Where to write the public key (32 raw bytes).
        #[arg(long)]
        public: PathBuf,
    },
    /// Scaffold a new rule pack with starter rules.yaml and cases.yaml.
    Init {
        /// Directory to create. Defaults to current directory.
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Build { input, out } => cmd_build(input, out),
        Cmd::Lint { inputs } => cmd_lint(inputs),
        Cmd::Run { rule, input, engine } => cmd_run(rule, input, engine),
        Cmd::RunNative { rule, input } => cmd_run_native(rule, input),
        Cmd::Test { rule, cases } => cmd_test(rule, cases),
        Cmd::Sign { rule, key } => cmd_sign(rule, key),
        Cmd::Verify { rule, pubkey } => cmd_verify(rule, pubkey),
        Cmd::Keygen { secret, public } => cmd_keygen(secret, public),
        Cmd::Init { path } => cmd_init(path),
    }
}

fn cmd_build(input: PathBuf, out: Option<PathBuf>) -> Result<()> {
    let src = fs::read_to_string(&input)
        .with_context(|| format!("reading {}", input.display()))?;
    let program = iratxo_core::compile_yaml(&src)
        .with_context(|| format!("compiling {}", input.display()))?;
    let bytes = iratxo_core::encode(&program);
    let out_path = out.unwrap_or_else(|| input.with_extension("iratxo"));
    fs::write(&out_path, &bytes)
        .with_context(|| format!("writing {}", out_path.display()))?;
    println!("compiled {} ({} bytes, {} rules) -> {}",
        program.name, bytes.len(), program.rules.len(), out_path.display());
    Ok(())
}

fn cmd_lint(inputs: Vec<PathBuf>) -> Result<()> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut walk_errors: Vec<(PathBuf, String)> = Vec::new();
    for input in &inputs {
        match collect_yaml_files(input) {
            Ok(found) => {
                if found.is_empty() && input.is_dir() {
                    eprintln!("warning: no .yaml/.yml files under {}", input.display());
                }
                files.extend(found);
            }
            Err(e) => walk_errors.push((input.clone(), e)),
        }
    }

    // Stable, deterministic order so the failure list is reproducible.
    files.sort();
    files.dedup();

    let mut failed: Vec<(PathBuf, String)> = Vec::new();
    for (i, path) in files.iter().enumerate() {
        if i > 0 { println!(); }
        println!("== {} ==", path.display());
        match lint_one(path) {
            Ok(()) => {}
            Err(reason) => failed.push((path.clone(), reason)),
        }
    }

    let total = files.len();
    let bad = failed.len() + walk_errors.len();
    println!();
    if bad == 0 {
        println!("lint ok: {} file{} passed", total, if total == 1 { "" } else { "s" });
        return Ok(());
    }

    eprintln!("lint failed: {} file{} with errors (out of {} checked):",
        bad, if bad == 1 { "" } else { "s" }, total + walk_errors.len());
    for (path, reason) in &walk_errors {
        eprintln!("  - {}: {}", path.display(), reason);
    }
    for (path, reason) in &failed {
        eprintln!("  - {}: {}", path.display(), reason);
    }
    std::process::exit(1);
}

/// Lint a single file. On success, prints the verbose per-rule summary to
/// stdout and returns Ok. On failure, prints the multi-line error display
/// to stderr and returns Err with a one-line reason for the final summary.
fn lint_one(path: &std::path::Path) -> std::result::Result<(), String> {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", path.display(), e);
            return Err(format!("read error: {}", e));
        }
    };

    let program = match iratxo_core::compile_yaml(&src) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: lint failed for {}", path.display());
            // DslError's Display can be multi-line (snippet + caret). Indent
            // each line so the file-path header stays visually distinct.
            let full = e.to_string();
            for line in full.lines() {
                eprintln!("  {}", line);
            }
            // First line of the error display is the most useful one-liner
            // for the trailing failed-files summary.
            let one_line = full.lines().next().unwrap_or("lint failed").to_string();
            return Err(one_line);
        }
    };

    let n = program.rules.len();
    println!("ok: {} ({} rule{})", program.name, n, if n == 1 { "" } else { "s" });
    if !program.description.is_empty() {
        println!("  description: {}", program.description);
    }
    let id_w = program.rules.iter().map(|r| r.id.len()).max().unwrap_or(0);
    let kind_w = program.rules.iter().map(|r| predicate_summary(&r.when).len()).max().unwrap_or(0);
    for r in &program.rules {
        let kind = predicate_summary(&r.when);
        let chain = if r.then.is_empty() {
            String::new()
        } else {
            format!(" -> [{}]", r.then.join(", "))
        };
        let exp = match &r.verdict.explanation {
            Some(s) if !s.is_empty() => format!(" \"{}\"", s),
            _ => String::new(),
        };
        println!(
            "  - {:<id_w$}  when {:<kind_w$}  => {} (conf {}){}{}",
            r.id, kind, r.verdict.classify, r.verdict.confidence, chain, exp,
            id_w = id_w, kind_w = kind_w,
        );
    }
    println!("  default: {} (conf {})", program.default.classify, program.default.confidence);
    Ok(())
}

/// Resolve a user-supplied path into the set of YAML rule files it covers.
/// Files are taken as-is. Directories are walked recursively; only entries
/// ending in `.yaml` or `.yml` are returned.
fn collect_yaml_files(path: &std::path::Path) -> std::result::Result<Vec<PathBuf>, String> {
    let meta = fs::metadata(path).map_err(|e| format!("cannot stat: {}", e))?;
    if meta.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !meta.is_dir() {
        return Err("not a file or directory".into());
    }
    let mut out = Vec::new();
    walk_dir(path, &mut out).map_err(|e| format!("walk error: {}", e))?;
    Ok(out)
}

fn walk_dir(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            walk_dir(&p, out)?;
        } else if ft.is_file() {
            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
                    out.push(p);
                }
            }
        }
    }
    Ok(())
}

/// One-line label of a predicate, for the verbose lint summary. Counts are
/// included where they help spot accidentally-empty lists.
fn predicate_summary(p: &iratxo_core::Predicate) -> String {
    use iratxo_core::Predicate as P;
    match p {
        P::ContainsAny { needles, .. }    => format!("contains_any({})", needles.len()),
        P::ContainsAll { needles, .. }    => format!("contains_all({})", needles.len()),
        P::NotContainsAny { needles, .. } => format!("not_contains_any({})", needles.len()),
        P::Regex { .. }                   => "regex".into(),
        P::MinLength { tokens }           => format!("min_length({})", tokens),
        P::MaxLength { tokens }           => format!("max_length({})", tokens),
        P::SemanticMatch(data) => format!("semantic_match({})", data.examples.len()),
        P::HasSection { titles }          => format!("has_section({})", titles.len()),
        P::HasEntity { kind, min_count }  => format!("has_entity({:?},{})", kind, min_count),
        P::ParagraphCount { .. }          => "paragraphs".into(),
        P::MaxWordsPerSentence { max }    => format!("max_words_per_sentence({})", max),
        P::LanguageIs { codes }           => format!("language_is({})", codes.join(",")),
        P::HasUrlToDomain { domains, .. } => format!("has_url_to_domain({})", domains.len()),
        P::MostlyUppercase { .. }         => "mostly_uppercase".into(),
        P::TokenEntropyAbove { .. }       => "token_entropy_above".into(),
        P::WordContainsAny { needles, .. } => format!("word_contains_any({})", needles.len()),
        P::StartsWithAny { prefixes, .. } => format!("starts_with_any({})", prefixes.len()),
        P::EndsWithAny { suffixes, .. }   => format!("ends_with_any({})", suffixes.len()),
        P::SentenceCount { .. }           => "sentences".into(),
        P::CharCount { .. }               => "chars".into(),
        P::LineCount { .. }               => "lines".into(),
        P::DigitRatioAbove { .. }         => "digit_ratio_above".into(),
        P::PunctuationRatioAbove { .. }   => "punctuation_ratio_above".into(),
        P::RepeatedCharRun { .. }         => "repeated_char_run".into(),
        P::RepeatedToken { .. }           => "repeated_token".into(),
        P::TypeTokenRatioBelow { .. }     => "type_token_ratio_below".into(),
        P::HasInvisibleChars              => "has_invisible_chars".into(),
        P::HasMixedScriptToken            => "has_mixed_script_token".into(),
        P::ScriptIs { scripts }           => format!("script_is({})", scripts.join(",")),
        P::All(items)                     => format!("all({})", items.len()),
        P::Any(items)                     => format!("any({})", items.len()),
        P::Not(_)                         => "not".into(),
        P::Always                         => "always".into(),
    }
}

fn default_engine_path() -> PathBuf {
    // Workspace layout: crates/iratxo-cli is the bin; engine artifact lives at
    // crates/iratxo-engine/target/wasm32-unknown-unknown/release/iratxo_engine.wasm
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent().unwrap()
        .join("iratxo-engine/target/wasm32-unknown-unknown/release/iratxo_engine.wasm")
}

fn cmd_run(rule: PathBuf, input: PathBuf, engine_path: Option<PathBuf>) -> Result<()> {
    let rule_bytes = fs::read(&rule)
        .with_context(|| format!("reading {}", rule.display()))?;
    let input_text = fs::read_to_string(&input)
        .with_context(|| format!("reading {}", input.display()))?;
    let engine_wasm = engine_path.unwrap_or_else(default_engine_path);
    let json = run_in_wasm(&engine_wasm, &rule_bytes, input_text.as_bytes())?;
    println!("{}", json);
    Ok(())
}

fn cmd_run_native(rule: PathBuf, input: PathBuf) -> Result<()> {
    let rule_bytes = fs::read(&rule)?;
    let input_text = fs::read_to_string(&input)?;
    let program = iratxo_core::decode(&rule_bytes)
        .map_err(|e| anyhow::anyhow!("decode: {}", e))?;
    let result = iratxo_core::evaluate(&program, &input_text);
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Deserialize)]
struct TestSuite {
    cases: Vec<TestCase>,
}

#[derive(Deserialize)]
struct TestCase {
    name: String,
    input: String,
    expect: TestExpect,
}

#[derive(Deserialize, Default)]
struct TestExpect {
    #[serde(default)]
    classify: Option<String>,
    /// Rule IDs that must be in the triggered list, order-independent.
    #[serde(default)]
    triggers: Vec<String>,
    /// Rule IDs that must NOT trigger.
    #[serde(default)]
    not_triggers: Vec<String>,
    /// Confidence range: triggered confidence must be within [min, max].
    #[serde(default)]
    confidence_min: Option<f32>,
    #[serde(default)]
    confidence_max: Option<f32>,
}

fn cmd_test(rule: PathBuf, cases: PathBuf) -> Result<()> {
    let rule_src = fs::read_to_string(&rule)?;
    let program = iratxo_core::compile_yaml(&rule_src)
        .map_err(|e| anyhow!("compile {}: {}", rule.display(), e))?;
    let cases_src = fs::read_to_string(&cases)?;
    let suite: TestSuite = serde_yaml::from_str(&cases_src)
        .with_context(|| format!("parsing {}", cases.display()))?;

    let mut passed = 0;
    let mut failed = 0;
    for case in suite.cases.iter() {
        let result = iratxo_core::evaluate(&program, &case.input);
        let mut errs: Vec<String> = Vec::new();
        if let Some(want) = &case.expect.classify {
            if &result.classification != want {
                errs.push(format!("classification = {:?}, want {:?}", result.classification, want));
            }
        }
        let triggered_ids: Vec<&str> = result.triggered.iter().map(|t| t.id.as_str()).collect();
        for need in &case.expect.triggers {
            if !triggered_ids.iter().any(|id| id == need) {
                errs.push(format!("expected trigger {:?} missing (got {:?})", need, triggered_ids));
            }
        }
        for forbid in &case.expect.not_triggers {
            if triggered_ids.iter().any(|id| id == forbid) {
                errs.push(format!("rule {:?} should not have triggered", forbid));
            }
        }
        if let Some(min) = case.expect.confidence_min {
            if result.confidence < min {
                errs.push(format!("confidence {} < min {}", result.confidence, min));
            }
        }
        if let Some(max) = case.expect.confidence_max {
            if result.confidence > max {
                errs.push(format!("confidence {} > max {}", result.confidence, max));
            }
        }

        if errs.is_empty() {
            println!("ok   - {}", case.name);
            passed += 1;
        } else {
            println!("FAIL - {}", case.name);
            for e in errs { println!("       {}", e); }
            failed += 1;
        }
    }
    println!("\n{} passed, {} failed", passed, failed);
    if failed > 0 { std::process::exit(1); }
    Ok(())
}

// ---------- signing ----------

const SIG_EXT: &str = "iratxo.sig";


const STARTER_RULE: &str = r#"name: my_first_pack
description: A starter compliance pack.

rules:
  - id: forbidden_phrase
    when:
      contains_any:
        - "guaranteed refund"
        - "100% refund"
    classify: review_required
    confidence: 0.95
    explanation: "Refund guarantee language detected."

  - id: medical_claim
    when:
      regex: "\\b(FDA approved|guaranteed cure)\\b"
    classify: blocked
    confidence: 0.99
    explanation: "Regulated medical claim detected."

default:
  classify: ok
  confidence: 1.0
"#;

const STARTER_CASES: &str = r#"cases:
  - name: clean message classifies ok
    input: "Hi there. This is a normal message."
    expect:
      classify: ok

  - name: refund guarantee triggers review
    input: "Sign up today for a guaranteed refund."
    expect:
      classify: review_required
      triggers: [forbidden_phrase]

  - name: medical claim triggers blocked
    input: "Our product is FDA approved."
    expect:
      classify: blocked
      triggers: [medical_claim]
      confidence_min: 0.99
"#;

fn cmd_init(path: Option<PathBuf>) -> Result<()> {
    let base = path.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&base)?;

    let rules_path = base.join("rules.yaml");
    let cases_path = base.join("cases.yaml");

    if rules_path.exists() {
        return Err(anyhow!("{} already exists", rules_path.display()));
    }
    if cases_path.exists() {
        return Err(anyhow!("{} already exists", cases_path.display()));
    }

    fs::write(&rules_path, STARTER_RULE)?;
    fs::write(&cases_path, STARTER_CASES)?;

    println!("created {} ({} bytes)", rules_path.display(), STARTER_RULE.len());
    println!("created {} ({} bytes)", cases_path.display(), STARTER_CASES.len());
    println!("\nNext steps:");
    println!("  iratxo lint {}", rules_path.display());
    println!("  iratxo test {} {}", rules_path.display(), cases_path.display());
    println!("  iratxo build {}", rules_path.display());
    Ok(())
}

fn cmd_keygen(secret: PathBuf, public: PathBuf) -> Result<()> {
    use rand::rngs::OsRng;
    let signing = SigningKey::generate(&mut OsRng);
    fs::write(&secret, signing.to_bytes())?;
    fs::write(&public, signing.verifying_key().to_bytes())?;
    // Best-effort: tighten permissions on the secret key on Unix.
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        let perm = std::fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(&secret, perm);
    }
    println!("wrote secret={} public={}", secret.display(), public.display());
    Ok(())
}

fn cmd_sign(rule: PathBuf, key: PathBuf) -> Result<()> {
    let bytes = fs::read(&rule)?;
    let secret_bytes = fs::read(&key)?;
    if secret_bytes.len() != 32 {
        return Err(anyhow!("secret key must be 32 raw bytes; got {}", secret_bytes.len()));
    }
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&secret_bytes);
    let signing = SigningKey::from_bytes(&buf);
    let sig = signing.sign(&bytes);
    let sig_path = rule.with_extension(SIG_EXT);
    fs::write(&sig_path, sig.to_bytes())?;
    println!("signed {} -> {}", rule.display(), sig_path.display());
    Ok(())
}

fn cmd_verify(rule: PathBuf, pubkey: PathBuf) -> Result<()> {
    let bytes = fs::read(&rule)?;
    let sig_path = rule.with_extension(SIG_EXT);
    let sig_bytes = fs::read(&sig_path)
        .with_context(|| format!("reading signature {}", sig_path.display()))?;
    if sig_bytes.len() != 64 {
        return Err(anyhow!("signature must be 64 bytes; got {}", sig_bytes.len()));
    }
    let mut sb = [0u8; 64];
    sb.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&sb);
    let pub_bytes = fs::read(&pubkey)?;
    if pub_bytes.len() != 32 {
        return Err(anyhow!("public key must be 32 bytes; got {}", pub_bytes.len()));
    }
    let mut pb = [0u8; 32];
    pb.copy_from_slice(&pub_bytes);
    let verifying = VerifyingKey::from_bytes(&pb)?;
    verifying.verify(&bytes, &sig)
        .map_err(|_| anyhow!("signature verification FAILED"))?;
    println!("ok: {} signature is valid", rule.display());
    Ok(())
}

/// Load the engine wasm, copy `rule` and `input` into its linear memory, call
/// `iratxo_execute`, then read the resulting JSON buffer back out.
pub fn run_in_wasm(engine_wasm_path: &std::path::Path, rule: &[u8], input: &[u8]) -> Result<String> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, engine_wasm_path)
        .with_context(|| format!("loading {}", engine_wasm_path.display()))?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;

    let memory = instance.get_memory(&mut store, "memory")
        .context("engine wasm exports no `memory`")?;
    let alloc = instance.get_typed_func::<u32, u32>(&mut store, "iratxo_alloc")?;
    let dealloc = instance.get_typed_func::<(u32, u32), ()>(&mut store, "iratxo_dealloc")?;
    let execute = instance.get_typed_func::<(u32, u32, u32, u32), u64>(&mut store, "iratxo_execute")?;

    let rule_ptr = alloc.call(&mut store, rule.len() as u32)?;
    memory.write(&mut store, rule_ptr as usize, rule)?;
    let input_ptr = alloc.call(&mut store, input.len() as u32)?;
    memory.write(&mut store, input_ptr as usize, input)?;

    let packed = execute.call(&mut store, (rule_ptr, rule.len() as u32, input_ptr, input.len() as u32))?;
    let out_ptr = (packed >> 32) as u32;
    let out_len = (packed & 0xFFFF_FFFF) as u32;

    let mut buf = vec![0u8; out_len as usize];
    memory.read(&store, out_ptr as usize, &mut buf)?;

    // Free everything we allocated in the guest.
    dealloc.call(&mut store, (rule_ptr, rule.len() as u32))?;
    dealloc.call(&mut store, (input_ptr, input.len() as u32))?;
    dealloc.call(&mut store, (out_ptr, out_len))?;

    Ok(String::from_utf8(buf)?)
}
