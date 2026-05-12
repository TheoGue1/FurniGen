import { useEffect, useState } from "react";
import { WardrobePreviewViewer } from "./components/WardrobePreviewViewer";
import { loadFurnigenWasm } from "./lib/wasm/load-furnigen-wasm";
import { wardrobeSpecSchema, type WardrobeSpec } from "./lib/spec/wardrobe-spec";

type WasmStatus = "loading" | "ready" | "error";

type InteriorUiMode = "none" | "equal_spacing";

function parseOptionalShelfThicknessMm(raw: string): number | undefined {
  const t = raw.trim();
  if (t === "") {
    return undefined;
  }
  const v = Number(t);
  if (!Number.isFinite(v) || v <= 0) {
    return undefined;
  }
  return v;
}

export function App() {
  const [wasmStatus, setWasmStatus] = useState<WasmStatus>("loading");
  const [wasmDetail, setWasmDetail] = useState<string>("");
  const [previewMeshJson, setPreviewMeshJson] = useState<string | null>(null);
  const [dims, setDims] = useState({
    width_mm: 2400,
    height_mm: 2200,
    depth_mm: 600,
    interiorMode: "equal_spacing" satisfies InteriorUiMode as InteriorUiMode,
    shelf_count: 4,
    /** Empty string = use core default (18 mm); otherwise positive mm sent as `shelf_thickness_mm`. */
    shelf_thickness_mm_str: "",
  });

  useEffect(() => {
    let cancelled = false;
    setWasmStatus("loading");
    setWasmDetail("");
    setPreviewMeshJson(null);

    (async () => {
      try {
        const wasm = await loadFurnigenWasm();
        if (cancelled) {
          return;
        }
        setWasmStatus("ready");
        setWasmDetail(`WASM ${wasm.wasmVersion()} loaded`);
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

  useEffect(() => {
    if (wasmStatus !== "ready") {
      return;
    }

    let cancelled = false;

    (async () => {
      try {
        const wasm = await loadFurnigenWasm();
        const shelfThicknessMm = parseOptionalShelfThicknessMm(dims.shelf_thickness_mm_str);
        const interior: WardrobeSpec["interior"] | undefined =
          dims.interiorMode === "equal_spacing"
            ? {
                type: "equal_spacing_shelves",
                shelf_count: Math.max(1, Math.floor(dims.shelf_count)),
                ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
              }
            : undefined;
        const spec: WardrobeSpec = {
          version: 1,
          layout: {
            type: "straight_run",
            width_mm: dims.width_mm,
            height_mm: dims.height_mm,
            depth_mm: dims.depth_mm,
          },
          interior,
          extensions: {},
        };
        wardrobeSpecSchema.parse(spec);
        const specText = JSON.stringify(spec);
        wasm.validateWardrobeSpecJson(specText);
        wasm.validateDepthMm(dims.depth_mm);
        const meshJson = wasm.buildWardrobePreviewMeshJson(specText);
        if (cancelled) {
          return;
        }
        setPreviewMeshJson(meshJson);
        const interiorLabel =
          dims.interiorMode === "none"
            ? "no interior"
            : `${Math.max(1, Math.floor(dims.shelf_count))} shelf boards (equal spacing)${
                shelfThicknessMm !== undefined ? ` · shelf ${shelfThicknessMm} mm thick` : ""
              }`;
        setWasmDetail(
          `WASM ${wasm.wasmVersion()} · ${Math.round(dims.width_mm)}×${Math.round(dims.height_mm)}×${Math.round(dims.depth_mm)} mm · ${interiorLabel} · WardrobeSpec v1 + preview mesh`
        );
      } catch (err) {
        if (cancelled) {
          return;
        }
        setPreviewMeshJson(null);
        const msg = err instanceof Error ? err.message : String(err);
        setWasmDetail(`WASM loaded · invalid dimensions: ${msg}`);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [
    wasmStatus,
    dims.width_mm,
    dims.height_mm,
    dims.depth_mm,
    dims.interiorMode,
    dims.shelf_count,
    dims.shelf_thickness_mm_str,
  ]);

  const meshForViewer = wasmStatus === "ready" && previewMeshJson ? previewMeshJson : null;

  return (
    <main
      className="flex min-h-screen flex-col items-center gap-6 p-8 pb-12"
      data-testid="app-root"
    >
      <div className="flex max-w-4xl flex-col items-center gap-3 text-center">
        <h1 className="text-2xl font-semibold tracking-tight">FurniGen</h1>
        <p className="text-slate-400">
          Parametric wardrobe preview (Rust core + WASM + Three.js). Spec and mesh use millimeters;
          the viewer uses WASM output only.
        </p>
        <form
          className="flex w-full max-w-xl flex-wrap items-end justify-center gap-4 rounded-lg border border-slate-700/80 bg-slate-950/30 px-4 py-3 text-left"
          onSubmit={(e) => e.preventDefault()}
          aria-label="Straight run dimensions (mm)"
        >
          <label className="flex min-w-[9rem] flex-col gap-1 text-xs text-slate-400">
            Interior
            <select
              data-testid="select-interior-mode"
              className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
              value={dims.interiorMode}
              onChange={(e) => {
                const v = e.target.value as InteriorUiMode;
                setDims((d) => ({ ...d, interiorMode: v }));
              }}
            >
              <option value="none">None</option>
              <option value="equal_spacing">Equal spacing shelves</option>
            </select>
          </label>
          <label className="flex min-w-[7.5rem] flex-col gap-1 text-xs text-slate-400">
            Width (mm)
            <input
              data-testid="input-width-mm"
              className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
              type="number"
              min={1}
              step={1}
              value={dims.width_mm}
              onChange={(e) => {
                const v = e.target.valueAsNumber;
                if (!Number.isFinite(v) || v <= 0) {
                  return;
                }
                setDims((d) => ({ ...d, width_mm: v }));
              }}
            />
          </label>
          <label className="flex min-w-[7.5rem] flex-col gap-1 text-xs text-slate-400">
            Height (mm)
            <input
              data-testid="input-height-mm"
              className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
              type="number"
              min={1}
              step={1}
              value={dims.height_mm}
              onChange={(e) => {
                const v = e.target.valueAsNumber;
                if (!Number.isFinite(v) || v <= 0) {
                  return;
                }
                setDims((d) => ({ ...d, height_mm: v }));
              }}
            />
          </label>
          <label className="flex min-w-[7.5rem] flex-col gap-1 text-xs text-slate-400">
            Depth (mm)
            <input
              data-testid="input-depth-mm"
              className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
              type="number"
              min={1}
              step={1}
              value={dims.depth_mm}
              onChange={(e) => {
                const v = e.target.valueAsNumber;
                if (!Number.isFinite(v) || v <= 0) {
                  return;
                }
                setDims((d) => ({ ...d, depth_mm: v }));
              }}
            />
          </label>
          {dims.interiorMode === "equal_spacing" && (
            <>
              <label className="flex min-w-[7.5rem] flex-col gap-1 text-xs text-slate-400">
                Shelf count
                <input
                  data-testid="input-shelf-count"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  step={1}
                  value={dims.shelf_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, shelf_count: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[10rem] flex-col gap-1 text-xs text-slate-400">
                Shelf thickness (mm, optional)
                <input
                  data-testid="input-shelf-thickness-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="text"
                  inputMode="decimal"
                  placeholder="default 18"
                  value={dims.shelf_thickness_mm_str}
                  onChange={(e) => {
                    setDims((d) => ({ ...d, shelf_thickness_mm_str: e.target.value }));
                  }}
                />
              </label>
            </>
          )}
        </form>
        <p className="text-sm text-slate-300" data-testid="wasm-status" aria-live="polite">
          {wasmStatus === "loading" && "Loading WASM…"}
          {wasmStatus === "ready" && wasmDetail}
          {wasmStatus === "error" && `WASM failed: ${wasmDetail}`}
        </p>
      </div>
      <WardrobePreviewViewer meshJson={meshForViewer} />
    </main>
  );
}
