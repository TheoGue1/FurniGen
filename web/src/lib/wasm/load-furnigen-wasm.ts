export type FurnigenWasmModule = typeof import("../../wasm/furnigen-wasm/furnigen_wasm.js");

let cache: FurnigenWasmModule | null = null;

/** Loads and initializes the wasm-pack `bundler` bundle once (Vite handles the `.wasm` asset). */
export async function loadFurnigenWasm(): Promise<FurnigenWasmModule> {
  if (cache) {
    return cache;
  }
  const mod = await import("../../wasm/furnigen-wasm/furnigen_wasm.js");
  await mod.default();
  cache = mod;
  return mod;
}
