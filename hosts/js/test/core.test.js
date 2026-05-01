// Smoke test for the shared `core.js` driver. Browser and Workers both reach
// the engine through this same code path with `WebAssembly.Module` they
// produce however they like (fetch / wasm binding). Running it through a
// manually compiled module here is equivalent.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { createFromModule } from "../src/core.js";

const here = dirname(fileURLToPath(import.meta.url));

async function freshIratxo() {
  const wasm = await readFile(join(here, "..", "wasm", "iratxo_engine.wasm"));
  const mod = await WebAssembly.compile(wasm);
  return createFromModule(mod);
}

test("core driver runs the engine", async () => {
  const iratxo = await freshIratxo();
  const rule = await readFile(join(here, "fixtures", "compliance.iratxo"));
  const r = iratxo.run(rule, "Sign up for a guaranteed refund. FDA approved.");
  assert.equal(r.classification, "blocked");
});

test("compile YAML to IR and run it", async () => {
  const iratxo = await freshIratxo();
  const yaml = `name: test_pack
rules:
  - id: hello
    when:
      contains_any: ["world"]
    classify: review_required
    confidence: 0.9
default:
  classify: ok
  confidence: 1.0`;
  const ir = iratxo.compile(yaml);
  assert.ok(ir instanceof Uint8Array);
  assert.ok(ir.length > 4);
  // IR starts with IRTX magic
  assert.equal(new TextDecoder().decode(ir.slice(0, 4)), "IRTX");

  const r = iratxo.run(ir, "hello world");
  assert.equal(r.classification, "review_required");
  assert.equal(r.triggered[0].id, "hello");

  const r2 = iratxo.run(ir, "nothing here");
  assert.equal(r2.classification, "ok");
});

test("compile rejects invalid YAML with clear error", async () => {
  const iratxo = await freshIratxo();
  assert.throws(() => iratxo.compile("not_yaml:::"), /parse error/);
});

test("rejects wasm without iratxo exports", async () => {
  // Hand-rolled wasm that exports nothing relevant.
  const wat = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, // \0asm
    0x01, 0x00, 0x00, 0x00, // version 1
  ]);
  const mod = await WebAssembly.compile(wat);
  const inst = await WebAssembly.instantiate(mod, {});
  const { createFromInstance } = await import("../src/core.js");
  assert.throws(() => createFromInstance(inst), /not an iratxo engine/);
});
