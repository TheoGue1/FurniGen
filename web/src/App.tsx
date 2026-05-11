import { useEffect, useState } from "react";
import minimalFixture from "../../spec-fixtures/wardrobe-spec-v1-minimal.json";
import { WardrobePreviewViewer } from "./components/WardrobePreviewViewer";
import { loadFurnigenWasm } from "./lib/wasm/load-furnigen-wasm";
import { wardrobeSpecSchema } from "./lib/spec/wardrobe-spec";

type WasmStatus = "loading" | "ready" | "error";

export function App() {
  const [wasmStatus, setWasmStatus] = useState<WasmStatus>("loading");
  const [wasmDetail, setWasmDetail] = useState<string>("");
  const [previewMeshJson, setPreviewMeshJson] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setWasmStatus("loading");
    setWasmDetail("");
    setPreviewMeshJson(null);

    (async () => {
      try {
        const wasm = await loadFurnigenWasm();
        wasm.validateDepthMm(600);
        const specText = JSON.stringify(minimalFixture);
        wardrobeSpecSchema.parse(JSON.parse(specText));
        wasm.validateWardrobeSpecJson(specText);
        const meshJson = wasm.buildWardrobePreviewMeshJson(specText);
        if (cancelled) {
          return;
        }
        setPreviewMeshJson(meshJson);
        setWasmDetail(
          `WASM ${wasm.wasmVersion()} · sample depth 600 mm validated · WardrobeSpec v1 + preview mesh`
        );
        setWasmStatus("ready");
      } catch (err) {
        if (cancelled) {
          return;
        }
        setPreviewMeshJson(null);
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
      className="flex min-h-screen flex-col items-center gap-6 p-8 pb-12"
      data-testid="app-root"
    >
      <div className="flex max-w-4xl flex-col items-center gap-3 text-center">
        <h1 className="text-2xl font-semibold tracking-tight">FurniGen</h1>
        <p className="text-slate-400">
          Parametric wardrobe preview (Rust core + WASM + Three.js). Spec and mesh use
          millimeters; the viewer uses WASM output only.
        </p>
        <p className="text-sm text-slate-300" data-testid="wasm-status" aria-live="polite">
          {wasmStatus === "loading" && "Loading WASM…"}
          {wasmStatus === "ready" && wasmDetail}
          {wasmStatus === "error" && `WASM failed: ${wasmDetail}`}
        </p>
      </div>
      <WardrobePreviewViewer meshJson={wasmStatus === "ready" ? previewMeshJson : null} />
    </main>
  );
}
