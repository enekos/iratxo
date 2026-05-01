# Iratxo Roadmap

The product thesis (from `docs/DESIGN.md`): **executable language rules as Wasm plugins** — deterministic, auditable, low-latency, runs anywhere. The MVP proves the core mechanic. Everything below is about turning that mechanic into something an organization will actually adopt.

This roadmap is opinionated. Items are ordered by impact-on-adoption, not difficulty. Each one ties back to the thesis.

---

## P0 — Now (next 1–2 sprints)

The current pitch is "runs anywhere via Wasm" but only `wasmtime` (a CLI host) is wired up. Until that changes, nobody can use this where they need to. Plus a few rule-authoring gaps that will bite the first real user.

### 1. Browser host (`iratxo-web`)
A 50–100 LOC JS package that loads `iratxo_engine.wasm` via `WebAssembly.instantiate`, exposes `iratxo.run(rule, input) -> Result`. Same ABI as wasmtime.
- **Why:** Browser execution is half the original pitch (CMS, IDE plugins, real-time form validation). Without it, "runs anywhere" is aspirational.
- **Risk:** None — the ABI is already JS-friendly (alloc/dealloc/execute, packed `(ptr<<32)|len` return).
- **Done when:** `npm i @iratxo/web`, demo HTML page, `cargo test` covers the JS↔Rust roundtrip with `wasm-bindgen-test`.

### 2. Node + Cloudflare Workers host
Same ABI, two thin wrappers. Workers in particular needs a zero-dep version.
- **Why:** The compliance and content-moderation use cases land on edge runtimes. Workers + Fastly Compute is where real traffic lives.
- **Done when:** `npm i @iratxo/node`, `wrangler dev` example. The wasm module has to stay below Workers' size limits — we're at 1.0M today, plenty of headroom but worth tracking.

### 3. `iratxo test` — golden-file rule testing
Rules are code. Code without tests rots. Add:
```
iratxo test rule.yaml --cases cases.yaml
```
where `cases.yaml` is `input: <text>` → `expect: { classify: review_required, triggers: [no_guarantee] }`.
- **Why:** Rule authoring is the user-facing surface. Without a test loop, nobody iterates with confidence.
- **Implementation note:** This is just a thin layer over `iratxo run-native` plus a YAML matcher. < 200 LOC.

### 4. Source-mapped error reporting in the YAML compiler
Today `compile_yaml` returns `serde_yaml::Error` with byte offsets. Wrap to produce `error at line 12: predicate must specify…` with a column pointer.
- **Why:** First impression of the DSL. Bad errors → users assume the tool is broken.

### 5. Per-language built-in synonym dictionaries (es, eu)
The English dict ships built in; Spanish and Basque users have to provide their own. Port a small `cancelar/terminar/rescindir`-class dictionary for each.
- **Why:** "Multilingual" is currently true for stemming but half-true for semantic matching. Pulling Spanish from a few open-source thesauri (CLEAR, RAE) and Basque from existing word-net data closes the loop.
- **Constraint:** Has to stay small (target <50KB per language) to keep the wasm tight.

---

## P1 — Next (1–2 months)

Adoption multipliers. None of these are MVPs on their own, but each one removes a barrier that would otherwise ceiling growth.

### 6. DSL: structural and entity predicates
Today: text predicates only. Real compliance rules need more:
- `must_contain_section: "Cancellation Policy"` — looks for headings in markdown/HTML.
- `extract_entity: { type: email | phone | url | currency, present: true, redact: false }`.
- `min_paragraphs`, `max_words_per_sentence` for readability checks.
- `language_must_be: en` (already detectable; just expose it).

### 7. Rule chaining / pipelines
Support `then: <rule_id>` so one rule's verdict can gate the next, and let multiple rules contribute to a final score (not just `max(confidence)`). Borrow from Stripe Radar's scorecard model.
- **Why:** Real policies are compositional. Today users have to fake this with `all`/`any`.

### 8. Versioning: `iratxo` IR schema versioning + engine compatibility
Add a 4-byte magic + version prefix to compiled `.iratxo` blobs. Engine refuses unknown versions with a clear error.
- **Why:** First real shipping moment will produce IR files that outlive the engine that built them. Without versioning, every refactor of the IR breaks every deployed rule.

### 9. Performance: regex compilation cache + bench harness
Currently `RegexBuilder::new(pattern).build()` runs on every `eval_predicate` call. Cache compiled regexes per `Predicate`. Add `criterion` benchmarks for representative rule packs.
- **Why:** The "low latency" claim has no evidence behind it yet. A real number ("evaluates 5K compliance rules over 10KB doc in <2ms") sells.
- **Stretch:** Compile the IR to a denser representation at load time (suffix-trie for `contains_any`, AC automaton for needle sets >10).

### 10. Fuzz harness for the interpreter
`cargo +nightly fuzz` over `(random_program, random_input)` checking: never panics, always terminates, deterministic across runs.
- **Why:** Determinism is a *product claim*. It needs proof, not vibes.

### 11. VS Code extension (preview-only, no LSP)
Syntax highlighting + hover preview that compiles the rule and shows the IR + lints. No language-server complexity yet.
- **Why:** The DSL is YAML; VS Code already does YAML poorly for nested predicates. A 200-LOC extension dramatically improves authoring.

---

## P2 — Later (3–6 months)

Ecosystem and durable moat. Don't start any of these before P0/P1 prove there's a user pulling for them.

### 12. Rule packs + signed registry
Bundle related rules (e.g. `@iratxo/compliance-fintech-us`) with metadata, version, and an Ed25519 signature. CLI verifies signatures before running. Registry can be a simple S3 bucket + JSON index initially — no central server needed.
- **Why:** This is the "npm for language governance" piece. It only works if there's already content worth distributing — that's why it's not P0.

### 13. Domain packs (paid path)
- `@iratxo/legal-termination-clauses`
- `@iratxo/medical-note-structure`
- `@iratxo/translation-qa-en-es`
- `@iratxo/marketing-claims-fda`

Each is a maintained YAML pack with golden test cases. This is also the most direct revenue path.

### 14. Observability: rule coverage + drift detection
When a rule is run in production, emit `{rule_id, triggered, confidence, input_hash}` to a sink. Build a small dashboard showing:
- Which rules have never triggered (probably stale).
- Which rules trigger >X% (probably too broad).
- When trigger rates drift week-over-week.

### 15. Embedded semantic dictionary upgrade
Today: hashing-trick + small synonym dict. Realistic accuracy ceiling. If users push for higher recall:
- Option A: ship a quantized 5K-word, 50-dim word vector table per language (~2MB each).
- Option B: defer to host-provided embeddings (Worker calls OpenAI/Anthropic, passes vectors in via a host import).
Decide based on actual user complaints — don't pre-build.

### 16. Web playground
Single page: paste rule YAML, paste input, see triggered rules + explanation in real time. Runs entirely in the browser via `iratxo-web`.
- **Why:** Best demo-to-conversion tool a developer-facing product can have.

---

## Maybe (worth thinking about, not committing to)

- **AI-assisted rule authoring.** "Show me 20 examples of customer messages requesting cancellation" → suggest a `semantic_match` rule. Tempting, but adds an LLM dep and erodes the "no LLM in the loop" pitch. Better as a *tool* alongside iratxo, not inside it.
- **WIT / Component Model migration.** The current C-ABI works fine; component model is more idiomatic but not load-bearing yet. Revisit when wasmtime/Workers/browsers all stabilize support.
- **Differential privacy for production logs.** Worth it if iratxo ends up in regulated environments where raw inputs can't leave the box.

---

## Anti-roadmap (explicit non-goals)

- **Generic NLP framework.** Iratxo is rules over text, not "do anything with text." Resist scope creep into POS tagging, dependency parsing, summarization.
- **Hosted SaaS as primary deployment.** The whole point is sovereignty and offline capability. SaaS would undercut the thesis.
- **Per-rule Wasm compilation.** Tempting because each rule could be smaller and possibly faster, but loses portability and the registry model. Universal interpreter wins.
- **Full Snowball stemmer roster.** Adding more languages should be demand-driven (one paying customer = one new stemmer), not catalogue-driven.

---

## Sequencing logic

P0 is exactly 5 items because that's roughly one engineer-month of work and produces a complete loop: author rule (improved errors) → test rule (`iratxo test`) → ship rule (browser/Node/Workers hosts) → match Spanish/Basque content (built-in dicts).

P1 is the productionization layer. Doing it before P0 is premature; skipping it means the first power user hits walls.

P2 is the moat. Skipping it means iratxo stays useful but not durable.

The decision worth making *now* is whether to start P0 in order (1 → 5), or to parallelize browser + `iratxo test` (the two highest-leverage items). I'd parallelize.
