# Iratxo — Executable Language Rules as Wasm Plugins

## MVP scope
Compliance + validation engine. Users author YAML rules; CLI compiles them to a portable binary IR; a single Wasm engine module evaluates `(rule_ir, input) -> result` deterministically.

## Architecture
```
   YAML (DSL)               binary IR (.iratxo)        Wasm engine
[author writes] --build--> [portable artifact] --run--> [classification + confidence + explanations]
```

- **iratxo-core**: IR types (serde + bincode), YAML parser → IR, interpreter.
- **iratxo-engine**: cdylib targeting `wasm32-unknown-unknown`. Re-exports the interpreter through a stable ABI:
  - `alloc(len) -> *mut u8`
  - `dealloc(ptr, len)`
  - `execute(rule_ptr, rule_len, input_ptr, input_len) -> u64` (high32 = ptr, low32 = len of JSON result)
- **iratxo-cli**: `build` (yaml → ir), `run` (wasmtime + ir + input), `lint` (validate yaml).

## DSL (MVP)
```yaml
name: forbidden_phrases
description: ...
input: { type: text }
rules:
  - id: no_guarantee
    when:
      contains_any: ["guaranteed refund", "100% refund"]
      case_sensitive: false
    classify: review_required
    confidence: 0.95
    explanation: "Forbidden refund guarantee phrase"
  - id: missing_disclaimer
    when:
      not_contains_any: ["This is not financial advice"]
    classify: review_required
    confidence: 0.8
default:
  classify: ok
  confidence: 1.0
```

Predicates supported in MVP: `contains_any`, `contains_all`, `not_contains_any`, `regex`, `min_length`, `max_length`, plus combinators `all`, `any`, `not`. Higher-confidence triggered rule wins; explanations from all triggered rules are returned.

## Determinism
No system time, no RNG, no I/O inside the engine. Pure `(rule, input) -> result`.

## Out of scope (MVP)
Semantic similarity (stubbed via token-overlap), multilingual dictionaries, marketplace, browser host, IDE plugin.
