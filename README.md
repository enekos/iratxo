# Iratxo

Executable language rules as Wasm plugins. Author rules in YAML, compile to a portable IR, evaluate them deterministically inside a single Wasm engine — same artifact runs in browsers, Node, edge runtimes, or backend services.

## Status

57 Rust tests + 850-row Basque parity test + 7 JS host tests all passing. Benchmarks: keyword pack 134µs, 6-regex pack over 5KB doc 246µs, semantic match 3.5µs (M-series Mac, release build).

## Components

- `iratxo-core` — IR, YAML→IR compiler, deterministic interpreter. Ships ~30 predicates spanning substring/regex matching, token & character shape gates, repetition / lexical-diversity heuristics, structural (heading, paragraph, sentence, line) gates, entity recognition (email, phone, URL, currency, IPv4/v6, Luhn-validated credit cards, IBAN, ISO dates, hashtags, mentions, emoji), Unicode-script + invisible-char homoglyph defenses, language detection (English / Spanish / Catalan / Basque / French / Italian / German / Dutch), and `semantic_match` (hashing-trick token vectors + per-language stemming + a built-in multilingual synonym dictionary; user-extensible).
- `iratxo-engine` — `cdylib` compiled to `wasm32-unknown-unknown`. Exports `iratxo_alloc`, `iratxo_dealloc`, `iratxo_execute`. Pure, no I/O, no time, no RNG.
- `iratxo-cli` — `iratxo build|lint|run|run-native|test|sign|verify|keygen`.
- `hosts/js` — JS package with three entry points (Node, browser, Cloudflare Workers) over the same wasm engine.
- `playground/` — browser-based rule studio: write YAML, compile, and evaluate in real time.
- `landing/` — Svelte 5 marketing + documentation site with an embedded live playground and 11 worked rule packs (compliance, legal, agent guardrails, support triage, UGC moderation, PII screen, multilingual, PR linter, phishing email, HIPAA PHI, secret scan). `cd landing && npm install && npm run dev`.
- `examples/` — compliance, legal/termination, phishing, HIPAA PHI, secret-scan domain packs, with golden-file test cases.

## Build

```
# core + cli (host)
cargo test -p iratxo-core
cargo build -p iratxo-cli

# wasm engine (Homebrew rustc lacks wasm32 std; use the rustup toolchain)
cd crates/iratxo-engine
~/.cargo/bin/cargo build --release --target wasm32-unknown-unknown
```

## Try it

```
# scaffold a new rule pack
./target/debug/iratxo init my_rules

# build, run, test
./target/debug/iratxo build examples/rules/compliance/forbidden_phrases.yaml
./target/debug/iratxo run   examples/rules/compliance/forbidden_phrases.iratxo \
                            examples/inputs/violating.txt
./target/debug/iratxo test  examples/rules/compliance/forbidden_phrases.yaml \
                            examples/cases/forbidden_phrases.cases.yaml

# sign + distribute
./target/debug/iratxo keygen --secret iratxo.sk --public iratxo.pk
./target/debug/iratxo sign   forbidden_phrases.iratxo --key iratxo.sk
./target/debug/iratxo verify forbidden_phrases.iratxo --pubkey iratxo.pk

# JS hosts
cd hosts/js && node --test test/

# Browser playground
python3 -m http.server  # then open /playground/index.html
```

Output of `iratxo run` for a violating input:

```json
{"classification":"blocked","confidence":0.99,"triggered":[...],"explanations":[...]}
```

## DSL — predicates

| predicate | what it checks |
|---|---|
| `contains_any` / `contains_all` / `not_contains_any` | substring membership |
| `regex` | regex match (cached per-evaluation) |
| `min_length` / `max_length` | token count bounds |
| `paragraphs: { min, max }` | paragraph count bounds |
| `max_words_per_sentence` | readability gate |
| `has_section: [...]` | markdown `#`-heading or `<h1>..<h6>` HTML heading |
| `has_entity: { kind, min_count }` | `email` / `phone` / `url` / `currency` |
| `language_is: [...]` | detected language ∈ codes (`en`, `es`, `ca`, `eu`, `fr`, `it`, `de`, `nl`) |
| `has_url_to_domain: { domains, allow_subdomains? }` | at least one URL host matches the allowlist (use `not:` for denylist) |
| `mostly_uppercase: { min_ratio }` | fraction of letters that are uppercase ≥ `min_ratio` |
| `token_entropy_above: { min_bits, min_token_len? }` | some whitespace-token of length ≥ `min_token_len` has Shannon entropy ≥ `min_bits` |
| `word_contains_any: [...]` | whole-word substring match (Unicode word boundaries) |
| `starts_with_any` / `ends_with_any: [...]` | prefix/suffix match against trimmed input |
| `sentences: { min, max }` / `chars: { min, max }` / `lines: { min, max }` | shape gates |
| `digit_ratio_above: { min_ratio }` / `punctuation_ratio_above: { min_ratio }` | character-class density gates |
| `repeated_char_run: { min_run }` | catches `soooooo`, `!!!!!`, etc. |
| `repeated_token: { min_count }` | same non-stopword token appears ≥ N times (copy-paste / spam) |
| `type_token_ratio_below: { max_ratio }` | low lexical diversity (unique / total tokens) |
| `has_invisible_chars` | zero-width / BOM-style chars (homoglyph / phishing) |
| `has_mixed_script_token` | a single token mixes Unicode scripts (e.g. Cyrillic 'а' inside Latin "PayPal") |
| `script_is: [...]` | input contains chars from any of: `latin`, `cyrillic`, `greek`, `han`, `hiragana`, `katakana`, `hangul`, `arabic`, `hebrew`, `devanagari`, `thai` |
| `has_entity: { kind, min_count }` | adds `ip_address`, `credit_card` (Luhn-validated), `iban`, `date_iso`, `hashtag`, `mention`, `emoji` |
| `semantic_match: { examples, threshold, language?, synonyms? }` | hashing-trick + per-language stem + synonym dict |
| `all` / `any` / `not` / `always` | combinators |

A rule may declare `then: [other_rule_id, ...]` to chain follow-up rules whose firing is gated on this rule triggering. Cycles are detected and ignored.

## DSL example

```yaml
name: outbound_compliance
rules:
  - id: no_refund_guarantee
    when:
      contains_any: ["guaranteed refund", "100% refund"]
    classify: review_required
    confidence: 0.95
    explanation: "Forbidden refund-guarantee language."
  - id: prohibited_claim
    when:
      regex: "\\b(cure|FDA approved)\\b"
    classify: blocked
    confidence: 0.99
  - id: cancellation_intent
    when:
      semantic_match:
        examples: ["the user wants to cancel their agreement"]
        threshold: 0.3
    classify: review_required
    confidence: 0.6
default:
  classify: ok
  confidence: 1.0
```

## Testing rules

```yaml
# cases.yaml
cases:
  - name: clean message classifies ok
    input: "Hi there. This is not financial advice."
    expect:
      classify: ok
  - name: FDA claim classifies blocked
    input: "Our product is FDA approved."
    expect:
      classify: blocked
      triggers: [prohibited_claim]
      confidence_min: 0.99
```

`iratxo test rule.yaml cases.yaml` exits 1 on any failure — wire it into CI.

## IR format

Compiled `.iratxo` files are versioned: `IRTX` (4 bytes) | version: `u16` LE | bincode payload. Engines refuse blobs with a different magic or version, so old artifacts produce a clear error rather than corrupted behavior. See `IR_VERSION` in `iratxo-core/src/lib.rs`. Current version: **5** (adds French, Italian, German, Dutch to `Language`; v4 boxed `SemanticMatch` for cache locality; v3 added Catalan to `Language`, new entity kinds — `ip_address`, `credit_card`, `iban`, `date_iso`, `hashtag`, `mention`, `emoji` — and 14 new heuristic predicates: `word_contains_any`, `starts_with_any`, `ends_with_any`, `sentences`, `chars`, `lines`, `digit_ratio_above`, `punctuation_ratio_above`, `repeated_char_run`, `repeated_token`, `type_token_ratio_below`, `has_invisible_chars`, `has_mixed_script_token`, `script_is`).

## Signing rule packs

```
iratxo keygen --secret iratxo.sk --public iratxo.pk   # 32 raw bytes each
iratxo sign   rule.iratxo --key iratxo.sk             # writes rule.iratxo.sig
iratxo verify rule.iratxo --pubkey iratxo.pk
```

Ed25519, 64-byte signature in a sidecar `.iratxo.sig`. Tampered blobs fail verification.

## Wasm ABI

```
iratxo_alloc(len: u32) -> ptr: u32
iratxo_dealloc(ptr: u32, len: u32)
iratxo_execute(rule_ptr, rule_len, input_ptr, input_len) -> u64   // (ptr<<32)|len of JSON
```

Any host that can read/write linear memory (browser JS, Node, Cloudflare Workers, wasmtime, wasmer) can run the engine with no engine-specific glue.

## Semantic match

Multilingual: English, Spanish, Catalan, Basque, French, Italian, German, Dutch. Auto-detected from input by default; specify `language` to force.

```yaml
- id: intencion_cancelar
  when:
    semantic_match:
      examples:
        - "el usuario quiere cancelar su contrato"
      threshold: 0.3
      language: "es"               # en | es | ca | eu | fr | it | de | nl | (omitted = auto)
      synonyms:                    # optional, layered over the built-in dict
        cancelar: ["terminar", "rescindir", "anular"]
        contrato: ["acuerdo", "convenio"]
  classify: cancelacion
  confidence: 0.9
```

**Pipeline:** tokenize (Unicode) → drop per-language stop words → stem → synonym-normalize → signed feature hashing into 256-dim dense vector → cosine similarity.

**Stemmers:**
- English / Spanish / French / Italian / German / Dutch — Snowball (via [`rust-stemmers`](https://crates.io/crates/rust-stemmers)).
- Catalan — hand-rolled light stemmer (`rust-stemmers` does not ship Catalan). Folds diacritics (`à è é í ï ò ó ú ü ç` → plain forms), strips the highest-frequency nominal/adjectival/verbal suffixes, and collapses orthographic `qu` → `c` so `polítiques` / `política` / `polítics` share a stem.
- Basque — direct port of [marrow](https://github.com/enekos/marrow)'s implementation, validated against marrow's reference output on 850 words (100% parity, see `crates/iratxo-core/tests/basque_parity.rs`).

**Score range:** 0.0–1.0. Realistic thresholds: 0.2 for sparse texts that share a single concept token after stemming, 0.3 for typical paragraph-length inputs, 0.4–0.5 for tight phrasing matches. Pure function, no embedded model file, no network.

## Hosts

```js
// Node
import { load } from "@iratxo/js";
const iratxo = await load();
const r = iratxo.run(ruleBytes, "input text");

// Browser
import { load } from "@iratxo/js/browser";
const iratxo = await load("/iratxo_engine.wasm");

// Compile YAML to IR in the browser
const ir = iratxo.compile(yamlString);
const result = iratxo.run(ir, "input text");

// Cloudflare Workers
import wasm from "./iratxo_engine.wasm";
import { loadFromModule } from "@iratxo/js/workers";
export default {
  async fetch(req, env) {
    const iratxo = loadFromModule(wasm);
    return Response.json(iratxo.run(env.RULE, await req.text()));
  }
};
```

All three share the same `core.js` driver; the engine wasm is identical across them.

## Roadmap

See `docs/ROADMAP.md`. Shipped from that document: source-mapped errors, IR versioning, regex caching, entity/structural/language/chaining predicates, browser+Node+Workers hosts, `iratxo test`, signing, criterion bench, no-panic property tests, built-in es/eu synonym dicts, legal-termination domain pack, web playground.

Still open: VS Code extension, observability dashboard, full domain pack catalog, per-rule wasm compilation if/when a use case emerges.
