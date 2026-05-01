//! Fuzz the IR decoder. A bad blob must produce a clean error, never a
//! panic. Engines run untrusted .iratxo files all the time — this is the
//! attack surface.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = iratxo_core::decode(data);
});
