//! Iratxo Wasm engine.
//!
//! Stable C ABI exposed to any Wasm host:
//!   - `iratxo_alloc(len: u32) -> *mut u8`
//!   - `iratxo_dealloc(ptr: *mut u8, len: u32)`
//!   - `iratxo_compile(yaml_ptr, yaml_len) -> u64`
//!     Returns `(ptr << 32) | len` for a JSON buffer:
//!     `{"ok":true,"ir":[1,2,3,...]}` on success,
//!     `{"ok":false,"error":"..."}` on failure.
//!   - `iratxo_execute(rule_ptr, rule_len, input_ptr, input_len) -> u64`
//!     where the returned u64 packs `(ptr << 32) | len` for a JSON result
//!     buffer the host must free with `iratxo_dealloc`.

#![no_main]

use std::alloc::{alloc, dealloc, Layout};

#[no_mangle]
pub extern "C" fn iratxo_alloc(len: u32) -> *mut u8 {
    if len == 0 { return std::ptr::null_mut(); }
    let layout = Layout::from_size_align(len as usize, 1).unwrap();
    unsafe { alloc(layout) }
}

#[no_mangle]
pub extern "C" fn iratxo_dealloc(ptr: *mut u8, len: u32) {
    if ptr.is_null() || len == 0 { return; }
    let layout = Layout::from_size_align(len as usize, 1).unwrap();
    unsafe { dealloc(ptr, layout) }
}

#[no_mangle]
pub extern "C" fn iratxo_compile(yaml_ptr: *const u8, yaml_len: u32) -> u64 {
    let yaml_bytes = unsafe { std::slice::from_raw_parts(yaml_ptr, yaml_len as usize) };
    let yaml = std::str::from_utf8(yaml_bytes).unwrap_or("");

    let json = match iratxo_core::compile_yaml(yaml) {
        Ok(program) => {
            let ir = iratxo_core::encode(&program);
            serde_json::json!({ "ok": true, "ir": ir }).to_string()
        }
        Err(e) => {
            serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
        }
    };

    let bytes = json.into_bytes();
    let len = bytes.len() as u32;
    let ptr = iratxo_alloc(len);
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()); }
    ((ptr as u64) << 32) | (len as u64)
}

#[no_mangle]
pub extern "C" fn iratxo_execute(
    rule_ptr: *const u8,
    rule_len: u32,
    input_ptr: *const u8,
    input_len: u32,
) -> u64 {
    let rule_bytes = unsafe { std::slice::from_raw_parts(rule_ptr, rule_len as usize) };
    let input_bytes = unsafe { std::slice::from_raw_parts(input_ptr, input_len as usize) };
    let input = std::str::from_utf8(input_bytes).unwrap_or("");

    let json = match iratxo_core::decode(rule_bytes) {
        Ok(program) => {
            let result = iratxo_core::evaluate(&program, input);
            serde_json::to_vec(&result).unwrap_or_else(|_| b"{\"error\":\"serialize\"}".to_vec())
        }
        Err(e) => {
            let msg = format!("{{\"error\":\"decode: {}\"}}", e);
            msg.into_bytes()
        }
    };

    let len = json.len() as u32;
    let ptr = iratxo_alloc(len);
    unsafe { std::ptr::copy_nonoverlapping(json.as_ptr(), ptr, json.len()); }
    ((ptr as u64) << 32) | (len as u64)
}
