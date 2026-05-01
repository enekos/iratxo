// Browser host: `fetch` the wasm with streaming compilation when supported.
import { createFromModule } from "./core.js";

/**
 * Load the engine over HTTP. Pass a URL pointing at a hosted
 * `iratxo_engine.wasm`, or a Response object for advanced cases.
 *
 * Uses `WebAssembly.compileStreaming` when the response has the right
 * MIME type — falls back to ArrayBuffer compile otherwise.
 *
 * @param {string | URL | Response} src
 */
export async function load(src) {
  let response = src instanceof Response ? src : await fetch(src);
  if (!response.ok) {
    throw new Error(`failed to fetch engine wasm: ${response.status} ${response.statusText}`);
  }
  let module;
  if (typeof WebAssembly.compileStreaming === "function") {
    try {
      module = await WebAssembly.compileStreaming(response.clone());
    } catch (_) {
      // some servers serve wrong MIME — fall through to non-streaming.
    }
  }
  if (!module) {
    const bytes = await response.arrayBuffer();
    module = await WebAssembly.compile(bytes);
  }
  return createFromModule(module);
}
