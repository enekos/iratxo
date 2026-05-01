//! Fuzz the interpreter: a deterministic engine must never panic on any
//! `(program, input)` pair. We feed arbitrary YAML straight into compile +
//! evaluate; if it doesn't compile we drop the iteration.
//!
//! Run with: `cargo +nightly fuzz run interpret`

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else { return; };
    // Split the fuzz input into (yaml, sample) on the first `\x00`. If no
    // null byte, treat the whole thing as the YAML and use a constant input.
    let (yaml, input) = match s.find('\0') {
        Some(i) => (&s[..i], &s[i+1..]),
        None    => (s, "the user wants to cancel"),
    };
    if let Ok(program) = iratxo_core::compile_yaml(yaml) {
        let _ = iratxo_core::evaluate(&program, input);
    }
});
