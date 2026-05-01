// Pure ABI driver, no I/O. Each host module supplies a `WebAssembly.Module`
// (already compiled) and we instantiate + drive it the same way everywhere.
//
// The wasm module exports:
//   memory                — linear memory
//   iratxo_alloc(len)     — returns ptr
//   iratxo_dealloc(p, len)
//   iratxo_compile(yaml_p, yaml_len) -> u64 packed (ptr<<32)|len (JSON)
//   iratxo_execute(rule_p, rule_len, input_p, input_len) -> u64 packed (ptr<<32)|len

const enc = new TextEncoder();
const dec = new TextDecoder();

/**
 * Create a runner bound to an instantiated wasm module. One Iratxo instance
 * holds one wasm instance — JS wasm linear memory is not safe to share across
 * concurrent calls, so create one per worker thread/request.
 *
 * @param {WebAssembly.Instance} instance
 */
export function createFromInstance(instance) {
  const m = instance.exports;
  if (!m.memory || !m.iratxo_alloc || !m.iratxo_execute) {
    throw new Error("not an iratxo engine wasm: missing required exports");
  }

  function copyIn(bytes) {
    const ptr = m.iratxo_alloc(bytes.length);
    new Uint8Array(m.memory.buffer, ptr, bytes.length).set(bytes);
    return ptr;
  }

  function readOut(packed) {
    // BigInt packing: high 32 bits = ptr, low 32 = len.
    const ptr = Number(packed >> 32n);
    const len = Number(packed & 0xffffffffn);
    const buf = new Uint8Array(m.memory.buffer, ptr, len).slice();
    m.iratxo_dealloc(ptr, len);
    return dec.decode(buf);
  }

  /**
   * Compile a YAML rule string to a .iratxo binary IR.
   * @param {string} yaml  rule source in YAML DSL
   * @returns {Uint8Array} compiled IR bytes
   * @throws {Error} on compilation failure
   */
  function compile(yaml) {
    const yamlBytes = enc.encode(yaml);
    const yamlPtr = copyIn(yamlBytes);
    const packed = m.iratxo_compile(yamlPtr, yamlBytes.length);
    const json = readOut(packed);
    m.iratxo_dealloc(yamlPtr, yamlBytes.length);
    const result = JSON.parse(json);
    if (!result.ok) {
      throw new Error(result.error);
    }
    return new Uint8Array(result.ir);
  }

  /**
   * Run a compiled .iratxo rule against an input string.
   * @param {Uint8Array} ruleBytes  raw .iratxo IR
   * @param {string}     input       text to classify
   * @returns {object} EvalResult
   */
  function run(ruleBytes, input) {
    const inputBytes = enc.encode(input);
    const rulePtr  = copyIn(ruleBytes);
    const inputPtr = copyIn(inputBytes);
    const packed = m.iratxo_execute(
      rulePtr, ruleBytes.length,
      inputPtr, inputBytes.length,
    );
    const json = readOut(packed);
    m.iratxo_dealloc(rulePtr,  ruleBytes.length);
    m.iratxo_dealloc(inputPtr, inputBytes.length);
    return JSON.parse(json);
  }

  return { compile, run };
}

/**
 * Convenience: take a `WebAssembly.Module` and return a ready Iratxo runner.
 */
export async function createFromModule(module) {
  const instance = await WebAssembly.instantiate(module, {});
  return createFromInstance(instance);
}
