import { useEffect, useState } from "react";
import minimalFixture from "../../spec-fixtures/wardrobe-spec-v1-minimal.json";
import { loadFurnigenWasm } from "./lib/wasm/load-furnigen-wasm";
import { wardrobeSpecSchema } from "./lib/spec/wardrobe-spec";

type WasmStatus = "loading" | "ready" | "error";

export function App() {
  const [wasmStatus, setWasmStatus] = useState<WasmStatus>("loading");
  const [wasmDetail, setWasmDetail] = useState<string>("");

  useEffect(() => {
    let cancelled = false;
    setWasmStatus("loading");
    setWasmDetail("");

    (async () => {
      try {
        const wasm = await loadFurnigenWasm();
        wasm.validateDepthMm(600);
        const specText = JSON.stringify(minimalFixture);
        wardrobeSpecSchema.parse(JSON.parse(specText));
        wasm.validateWardrobeSpecJson(specText);
        if (cancelled) {
          return;
        }
        setWasmDetail(
          `WASM ${wasm.wasmVersion()} · depth check + WardrobeSpec v1 golden validated`
        );
        setWasmStatus("ready");
      } catch (err) {
        if (cancelled) {
          return;
        }
        setWasmDetail(err instanceof Error ? err.message : String(err));
        setWasmStatus("error");
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main
      className="flex min-h-screen flex-col items-center justify-center gap-4 p-8"
      data-testid="app-root"
    >
      <h1 className="text-2xl font-semibold tracking-tight">FurniGen</h1>
      <p className="max-w-md text-center text-slate-400">
        Parametric wardrobe preview (Rust core + WASM + web). Units in the spec are
        millimeters.
      </p>
      <p
        className="max-w-md text-center text-sm text-slate-300"
        data-testid="wasm-status"
        aria-live="polite"
      >
        {wasmStatus === "loading" && "Loading WASM…"}
        {wasmStatus === "ready" && wasmDetail}
        {wasmStatus === "error" && `WASM failed: ${wasmDetail}`}
      </p>
    </main>
  );
}
