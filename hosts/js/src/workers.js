// Cloudflare Workers (and Vercel Edge / Deno Deploy) host.
//
// On Workers, you import the .wasm at module-load via the `wasm` binding,
// which gives you a `WebAssembly.Module` directly. Pass that module here to
// get a runner. Instantiation is per-request because wasm memory is not
// shareable across concurrent invocations within a worker.
import { createFromModule } from "./core.js";

/**
 * @param {WebAssembly.Module} module already-compiled engine
 */
export function loadFromModule(module) {
  return createFromModule(module);
}

export { createFromInstance, createFromModule } from "./core.js";
