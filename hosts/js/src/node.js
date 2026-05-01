// Node host: loads wasm from disk via `fs/promises`.
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { createFromModule } from "./core.js";

const here = dirname(fileURLToPath(import.meta.url));
// Keep the wasm bundled alongside the package so consumers don't need to
// know where it lives.
const DEFAULT_WASM_PATH = join(here, "..", "wasm", "iratxo_engine.wasm");

/**
 * Load the engine and return an Iratxo runner.
 * @param {string} [wasmPath]
 */
export async function load(wasmPath = DEFAULT_WASM_PATH) {
  const bytes = await readFile(wasmPath);
  const module = await WebAssembly.compile(bytes);
  return createFromModule(module);
}
