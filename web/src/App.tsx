import { useEffect, useState } from "react";
import { WardrobePreviewViewer } from "./components/WardrobePreviewViewer";
import { loadFurnigenWasm } from "./lib/wasm/load-furnigen-wasm";
import { wardrobeSpecSchema, type WardrobeSpec } from "./lib/spec/wardrobe-spec";

type WasmStatus = "loading" | "ready" | "error";

type InteriorUiMode =
  | "none"
  | "equal_spacing"
  | "explicit_heights"
  | "zones_equal_fill"
  | "golden_ratio_ladder"
  | "two_tier_rhythm"
  | "max_shelves_min_segment"
  | "seeded_random_min_gap"
  | "weighted_random_band"
  | "equal_vertical_bays"
  | "grid_uprights_explicit";

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

function parseOptionalPositiveInt(raw: string): number | undefined {
  const t = raw.trim();
  if (t === "") {
    return undefined;
  }
  const v = Number(t);
  if (!Number.isFinite(v) || v < 1 || v > 500 || !Number.isInteger(v)) {
    return undefined;
  }
  return v;
}

/** Parses shelf bottom Y values (mm); one per line or comma-separated. Order preserved. */
function parseShelfBottomYListMm(raw: string): number[] {
  return raw
    .split(/[\s,;]+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
    .map((s) => Number(s))
    .filter((n) => Number.isFinite(n));
}

function parseOptionalNonNegativeNumber(raw: string): number | undefined {
  const t = raw.trim();
  if (t === "") {
    return undefined;
  }
  const v = Number(t);
  if (!Number.isFinite(v) || v < 0) {
    return undefined;
  }
  return v;
}

function buildClearanceFromDims(d: {
  clearance_panel_thickness_str: string;
  clearance_side_inset_str: string;
  clearance_front_setback_str: string;
  clearance_nosing_str: string;
}): WardrobeSpec["clearance"] {
  const carcass_panel_thickness_mm = parseOptionalNonNegativeNumber(d.clearance_panel_thickness_str);
  const side_inset_mm = parseOptionalNonNegativeNumber(d.clearance_side_inset_str);
  const front_setback_mm = parseOptionalNonNegativeNumber(d.clearance_front_setback_str);
  const shelf_nosing_mm = parseOptionalNonNegativeNumber(d.clearance_nosing_str);
  if (
    carcass_panel_thickness_mm === undefined &&
    side_inset_mm === undefined &&
    front_setback_mm === undefined &&
    shelf_nosing_mm === undefined
  ) {
    return undefined;
  }
  return {
    ...(carcass_panel_thickness_mm !== undefined ? { carcass_panel_thickness_mm } : {}),
    ...(side_inset_mm !== undefined ? { side_inset_mm } : {}),
    ...(front_setback_mm !== undefined ? { front_setback_mm } : {}),
    ...(shelf_nosing_mm !== undefined ? { shelf_nosing_mm } : {}),
  };
}

function parseOptionalMinGapMm(raw: string): number | undefined {
  const t = raw.trim();
  if (t === "") {
    return undefined;
  }
  const v = Number(t);
  if (!Number.isFinite(v) || v < 0) {
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
    /** For explicit shelf mode: one Y per line or comma-separated (mm from inner floor, ascending). */
    explicit_shelf_bottoms_y_str: "400\n1000\n1600",
    explicit_min_gap_mm_str: "",
    /** Reserved from inner floor / ceiling (mm) for zones + equal-fill mode. */
    zones_bottom_zone_mm: 500,
    zones_top_reserve_mm: 300,
    zones_shelf_count: 3,
    /** Horizontal shelf boards for golden-ratio ladder (φ-weighted air gaps). */
    golden_rungs: 3,
    /** Two-tier rhythm: transition Y (mm), dense gap below / wider gap at and above shelf tops crossing it. */
    two_tier_transition_y_mm: 900,
    two_tier_gap_lower_mm: 80,
    two_tier_gap_upper_mm: 200,
    two_tier_top_reserve_mm: 0,
    /** Max shelves / min vertical segment (mm); optional target count in `max_shelf_count_str`. */
    max_min_vertical_segment_mm: 100,
    max_bottom_reserve_mm: 0,
    max_top_reserve_mm: 0,
    /** Empty = maximum feasible shelf count from core. */
    max_shelf_count_str: "",
    seeded_seed_str: "999",
    seeded_shelf_count: 3,
    seeded_min_gap_mm: 80,
    seeded_bottom_reserve_mm: 100,
    seeded_top_reserve_mm: 100,
    weighted_seed_str: "42",
    weighted_shelf_count: 3,
    weighted_min_gap_mm: 64,
    weighted_bottom_reserve_mm: 80,
    weighted_top_reserve_mm: 80,
    weighted_band_lower: 3,
    weighted_band_middle: 1,
    weighted_band_upper: 1,
    bay_count: 3,
    bay_shelf_count: 4,
    upright_thickness_mm_str: "18",
    grid_bay_count: 2,
    grid_shelf_y_str: "500\n1200\n1700",
    clearance_panel_thickness_str: "",
    clearance_side_inset_str: "",
    clearance_front_setback_str: "",
    clearance_nosing_str: "",
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
            : dims.interiorMode === "explicit_heights"
              ? (() => {
                  const shelf_bottom_y_mm = parseShelfBottomYListMm(dims.explicit_shelf_bottoms_y_str);
                  const minGapMm = parseOptionalMinGapMm(dims.explicit_min_gap_mm_str);
                  return {
                    type: "explicit_shelf_heights" as const,
                    shelf_bottom_y_mm,
                    ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                    ...(minGapMm !== undefined ? { min_gap_mm: minGapMm } : {}),
                  };
                })()
              : dims.interiorMode === "zones_equal_fill"
                ? {
                    type: "zones_equal_fill_shelves" as const,
                    bottom_zone_mm: Math.max(0, dims.zones_bottom_zone_mm),
                    top_reserve_mm: Math.max(0, dims.zones_top_reserve_mm),
                    shelf_count: Math.max(1, Math.floor(dims.zones_shelf_count)),
                    ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                  }
                : dims.interiorMode === "golden_ratio_ladder"
                  ? {
                      type: "golden_ratio_ladder_shelves" as const,
                      rungs: Math.max(1, Math.floor(dims.golden_rungs)),
                      ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                    }
                  : dims.interiorMode === "two_tier_rhythm"
                    ? {
                        type: "two_tier_rhythm_shelves" as const,
                        transition_y_mm: dims.two_tier_transition_y_mm,
                        gap_lower_mm: dims.two_tier_gap_lower_mm,
                        gap_upper_mm: dims.two_tier_gap_upper_mm,
                        ...(dims.two_tier_top_reserve_mm > 0
                          ? { top_reserve_mm: Math.max(0, dims.two_tier_top_reserve_mm) }
                          : {}),
                        ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                      }
                    : dims.interiorMode === "max_shelves_min_segment"
                      ? (() => {
                          const target = parseOptionalPositiveInt(dims.max_shelf_count_str);
                          return {
                            type: "max_shelves_min_segment_shelves" as const,
                            min_vertical_segment_mm: Math.max(0.01, dims.max_min_vertical_segment_mm),
                            ...(dims.max_bottom_reserve_mm > 0
                              ? { bottom_reserve_mm: Math.max(0, dims.max_bottom_reserve_mm) }
                              : {}),
                            ...(dims.max_top_reserve_mm > 0
                              ? { top_reserve_mm: Math.max(0, dims.max_top_reserve_mm) }
                              : {}),
                            ...(target !== undefined ? { shelf_count: target } : {}),
                            ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                          };
                        })()
                      : dims.interiorMode === "seeded_random_min_gap"
                        ? {
                            type: "seeded_random_min_gap_shelves" as const,
                            seed: Math.max(0, Math.floor(Number(dims.seeded_seed_str)) || 0),
                            shelf_count: Math.max(1, Math.floor(dims.seeded_shelf_count)),
                            min_gap_mm: Math.max(0.01, dims.seeded_min_gap_mm),
                            ...(dims.seeded_bottom_reserve_mm > 0
                              ? { bottom_reserve_mm: dims.seeded_bottom_reserve_mm }
                              : {}),
                            ...(dims.seeded_top_reserve_mm > 0
                              ? { top_reserve_mm: dims.seeded_top_reserve_mm }
                              : {}),
                            ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                          }
                        : dims.interiorMode === "weighted_random_band"
                          ? {
                              type: "weighted_random_band_shelves" as const,
                              seed: Math.max(0, Math.floor(Number(dims.weighted_seed_str)) || 0),
                              shelf_count: Math.max(1, Math.floor(dims.weighted_shelf_count)),
                              min_gap_mm: Math.max(0.01, dims.weighted_min_gap_mm),
                              ...(dims.weighted_bottom_reserve_mm > 0
                                ? { bottom_reserve_mm: dims.weighted_bottom_reserve_mm }
                                : {}),
                              ...(dims.weighted_top_reserve_mm > 0
                                ? { top_reserve_mm: dims.weighted_top_reserve_mm }
                                : {}),
                              ...(dims.weighted_band_lower !== 1
                                ? { band_weight_lower: dims.weighted_band_lower }
                                : {}),
                              ...(dims.weighted_band_middle !== 1
                                ? { band_weight_middle: dims.weighted_band_middle }
                                : {}),
                              ...(dims.weighted_band_upper !== 1
                                ? { band_weight_upper: dims.weighted_band_upper }
                                : {}),
                              ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                            }
                          : dims.interiorMode === "equal_vertical_bays"
                            ? {
                                type: "equal_vertical_bays_equal_spacing_shelves" as const,
                                bay_count: Math.max(1, Math.floor(dims.bay_count)),
                                shelf_count: Math.max(1, Math.floor(dims.bay_shelf_count)),
                                ...(parseOptionalShelfThicknessMm(dims.upright_thickness_mm_str) !== undefined
                                  ? {
                                      upright_thickness_mm: parseOptionalShelfThicknessMm(
                                        dims.upright_thickness_mm_str,
                                      ),
                                    }
                                  : {}),
                                ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                              }
                            : dims.interiorMode === "grid_uprights_explicit"
                              ? {
                                  type: "grid_uprights_explicit_rows_shelves" as const,
                                  bay_count: Math.max(1, Math.floor(dims.grid_bay_count)),
                                  shelf_bottom_y_mm: parseShelfBottomYListMm(dims.grid_shelf_y_str),
                                  ...(parseOptionalShelfThicknessMm(dims.upright_thickness_mm_str) !== undefined
                                    ? {
                                        upright_thickness_mm: parseOptionalShelfThicknessMm(
                                          dims.upright_thickness_mm_str,
                                        ),
                                      }
                                    : {}),
                                  ...(shelfThicknessMm !== undefined ? { shelf_thickness_mm: shelfThicknessMm } : {}),
                                }
                              : undefined;
        const clearance = buildClearanceFromDims(dims);
        const spec: WardrobeSpec = {
          version: 1,
          layout: {
            type: "straight_run",
            width_mm: dims.width_mm,
            height_mm: dims.height_mm,
            depth_mm: dims.depth_mm,
          },
          interior,
          ...(clearance !== undefined ? { clearance } : {}),
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
            : dims.interiorMode === "equal_spacing"
              ? `${Math.max(1, Math.floor(dims.shelf_count))} shelf boards (equal spacing)${
                  shelfThicknessMm !== undefined ? ` · shelf ${shelfThicknessMm} mm thick` : ""
                }`
              : dims.interiorMode === "zones_equal_fill"
                ? `${Math.max(1, Math.floor(dims.zones_shelf_count))} shelf boards in middle band (bottom reserve ${Math.round(dims.zones_bottom_zone_mm)} mm · top reserve ${Math.round(dims.zones_top_reserve_mm)} mm)${
                    shelfThicknessMm !== undefined ? ` · shelf ${shelfThicknessMm} mm thick` : ""
                  }`
                : dims.interiorMode === "golden_ratio_ladder"
                  ? `${Math.max(1, Math.floor(dims.golden_rungs))} shelf boards (golden-ratio ladder gaps)${
                      shelfThicknessMm !== undefined ? ` · shelf ${shelfThicknessMm} mm thick` : ""
                    }`
                  : dims.interiorMode === "two_tier_rhythm"
                    ? `two-tier rhythm (transition ${Math.round(dims.two_tier_transition_y_mm)} mm · gaps ${Math.round(dims.two_tier_gap_lower_mm)}/${Math.round(dims.two_tier_gap_upper_mm)} mm)${
                        shelfThicknessMm !== undefined ? ` · shelf ${shelfThicknessMm} mm thick` : ""
                      }`
                    : dims.interiorMode === "max_shelves_min_segment"
                      ? `max shelves / min segment ${Math.round(dims.max_min_vertical_segment_mm)} mm${
                          parseOptionalPositiveInt(dims.max_shelf_count_str) !== undefined
                            ? ` · target count ${parseOptionalPositiveInt(dims.max_shelf_count_str)}`
                            : ""
                        }${shelfThicknessMm !== undefined ? ` · shelf ${shelfThicknessMm} mm thick` : ""}`
                      : dims.interiorMode === "seeded_random_min_gap"
                        ? `seeded random · ${Math.max(1, Math.floor(dims.seeded_shelf_count))} shelves · min gap ${Math.round(dims.seeded_min_gap_mm)} mm`
                        : dims.interiorMode === "weighted_random_band"
                          ? `weighted random bands · ${Math.max(1, Math.floor(dims.weighted_shelf_count))} shelves`
                          : dims.interiorMode === "equal_vertical_bays"
                            ? `${Math.max(1, Math.floor(dims.bay_count))} bays × ${Math.max(1, Math.floor(dims.bay_shelf_count))} shelf rows`
                            : dims.interiorMode === "grid_uprights_explicit"
                              ? `${Math.max(1, Math.floor(dims.grid_bay_count))} bays · explicit global rows`
                              : `${parseShelfBottomYListMm(dims.explicit_shelf_bottoms_y_str).length} explicit shelf bottom Y value(s)${
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
    dims.explicit_shelf_bottoms_y_str,
    dims.explicit_min_gap_mm_str,
    dims.zones_bottom_zone_mm,
    dims.zones_top_reserve_mm,
    dims.zones_shelf_count,
    dims.golden_rungs,
    dims.two_tier_transition_y_mm,
    dims.two_tier_gap_lower_mm,
    dims.two_tier_gap_upper_mm,
    dims.two_tier_top_reserve_mm,
    dims.max_min_vertical_segment_mm,
    dims.max_bottom_reserve_mm,
    dims.max_top_reserve_mm,
    dims.max_shelf_count_str,
    dims.seeded_seed_str,
    dims.seeded_shelf_count,
    dims.seeded_min_gap_mm,
    dims.seeded_bottom_reserve_mm,
    dims.seeded_top_reserve_mm,
    dims.weighted_seed_str,
    dims.weighted_shelf_count,
    dims.weighted_min_gap_mm,
    dims.weighted_bottom_reserve_mm,
    dims.weighted_top_reserve_mm,
    dims.weighted_band_lower,
    dims.weighted_band_middle,
    dims.weighted_band_upper,
    dims.bay_count,
    dims.bay_shelf_count,
    dims.upright_thickness_mm_str,
    dims.grid_bay_count,
    dims.grid_shelf_y_str,
    dims.clearance_panel_thickness_str,
    dims.clearance_side_inset_str,
    dims.clearance_front_setback_str,
    dims.clearance_nosing_str,
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
              <option value="explicit_heights">Explicit shelf bottom Y (mm)</option>
              <option value="zones_equal_fill">Zones + equal-fill (middle band)</option>
              <option value="golden_ratio_ladder">Golden ratio ladder shelves</option>
              <option value="two_tier_rhythm">Two-tier rhythm shelves</option>
              <option value="max_shelves_min_segment">Max shelves / min segment height</option>
              <option value="seeded_random_min_gap">Seeded random (min gap)</option>
              <option value="weighted_random_band">Weighted random (vertical bands)</option>
              <option value="equal_vertical_bays">Equal vertical bays + equal spacing</option>
              <option value="grid_uprights_explicit">Grid uprights + explicit rows</option>
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
          {(dims.interiorMode === "equal_spacing" ||
            dims.interiorMode === "explicit_heights" ||
            dims.interiorMode === "zones_equal_fill" ||
            dims.interiorMode === "golden_ratio_ladder" ||
            dims.interiorMode === "two_tier_rhythm" ||
            dims.interiorMode === "max_shelves_min_segment" ||
            dims.interiorMode === "seeded_random_min_gap" ||
            dims.interiorMode === "weighted_random_band" ||
            dims.interiorMode === "equal_vertical_bays" ||
            dims.interiorMode === "grid_uprights_explicit") && (
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
          )}
          {dims.interiorMode === "zones_equal_fill" && (
            <>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Bottom reserve (mm)
                <input
                  data-testid="input-zones-bottom-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  step={1}
                  value={dims.zones_bottom_zone_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, zones_bottom_zone_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Top reserve (mm)
                <input
                  data-testid="input-zones-top-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  step={1}
                  value={dims.zones_top_reserve_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, zones_top_reserve_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[7.5rem] flex-col gap-1 text-xs text-slate-400">
                Shelves in band
                <input
                  data-testid="input-zones-shelf-count"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  step={1}
                  value={dims.zones_shelf_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, zones_shelf_count: v }));
                  }}
                />
              </label>
            </>
          )}
          {dims.interiorMode === "golden_ratio_ladder" && (
            <label className="flex min-w-[7.5rem] flex-col gap-1 text-xs text-slate-400">
              Rungs (shelf count)
              <input
                data-testid="input-golden-rungs"
                className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                type="number"
                min={1}
                max={500}
                step={1}
                value={dims.golden_rungs}
                onChange={(e) => {
                  const v = e.target.valueAsNumber;
                  if (!Number.isFinite(v) || v < 1 || v > 500) {
                    return;
                  }
                  setDims((d) => ({ ...d, golden_rungs: v }));
                }}
              />
            </label>
          )}
          {dims.interiorMode === "two_tier_rhythm" && (
            <>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Transition Y (mm)
                <input
                  data-testid="input-two-tier-transition-y-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  step={1}
                  value={dims.two_tier_transition_y_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1) {
                      return;
                    }
                    setDims((d) => ({ ...d, two_tier_transition_y_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Lower gap (mm)
                <input
                  data-testid="input-two-tier-gap-lower-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  step={1}
                  value={dims.two_tier_gap_lower_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1) {
                      return;
                    }
                    setDims((d) => ({ ...d, two_tier_gap_lower_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Upper gap (mm)
                <input
                  data-testid="input-two-tier-gap-upper-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  step={1}
                  value={dims.two_tier_gap_upper_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1) {
                      return;
                    }
                    setDims((d) => ({ ...d, two_tier_gap_upper_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Top reserve (mm)
                <input
                  data-testid="input-two-tier-top-reserve-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  step={1}
                  value={dims.two_tier_top_reserve_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, two_tier_top_reserve_mm: v }));
                  }}
                />
              </label>
            </>
          )}
          {dims.interiorMode === "max_shelves_min_segment" && (
            <>
              <label className="flex min-w-[10rem] flex-col gap-1 text-xs text-slate-400">
                Min vertical segment (mm)
                <input
                  data-testid="input-max-min-segment-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  step={1}
                  value={dims.max_min_vertical_segment_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1) {
                      return;
                    }
                    setDims((d) => ({ ...d, max_min_vertical_segment_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Bottom reserve (mm)
                <input
                  data-testid="input-max-bottom-reserve-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  step={1}
                  value={dims.max_bottom_reserve_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, max_bottom_reserve_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Top reserve (mm)
                <input
                  data-testid="input-max-top-reserve-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  step={1}
                  value={dims.max_top_reserve_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, max_top_reserve_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[11rem] flex-col gap-1 text-xs text-slate-400">
                Target shelf count (optional)
                <input
                  data-testid="input-max-shelf-count-str"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 font-mono text-sm text-slate-100"
                  type="text"
                  inputMode="numeric"
                  placeholder="max feasible"
                  value={dims.max_shelf_count_str}
                  onChange={(e) => {
                    setDims((d) => ({ ...d, max_shelf_count_str: e.target.value }));
                  }}
                />
              </label>
            </>
          )}
          {dims.interiorMode === "seeded_random_min_gap" && (
            <>
              <label className="flex min-w-[7rem] flex-col gap-1 text-xs text-slate-400">
                Seed
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 font-mono text-sm text-slate-100"
                  type="text"
                  inputMode="numeric"
                  value={dims.seeded_seed_str}
                  onChange={(e) => setDims((d) => ({ ...d, seeded_seed_str: e.target.value }))}
                />
              </label>
              <label className="flex min-w-[7rem] flex-col gap-1 text-xs text-slate-400">
                Shelf count
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  value={dims.seeded_shelf_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, seeded_shelf_count: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Min gap (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0.01}
                  step={1}
                  value={dims.seeded_min_gap_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0.01) {
                      return;
                    }
                    setDims((d) => ({ ...d, seeded_min_gap_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Bottom reserve (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  value={dims.seeded_bottom_reserve_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, seeded_bottom_reserve_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Top reserve (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0}
                  value={dims.seeded_top_reserve_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, seeded_top_reserve_mm: v }));
                  }}
                />
              </label>
            </>
          )}
          {dims.interiorMode === "weighted_random_band" && (
            <>
              <label className="flex min-w-[7rem] flex-col gap-1 text-xs text-slate-400">
                Seed
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 font-mono text-sm text-slate-100"
                  type="text"
                  value={dims.weighted_seed_str}
                  onChange={(e) => setDims((d) => ({ ...d, weighted_seed_str: e.target.value }))}
                />
              </label>
              <label className="flex min-w-[7rem] flex-col gap-1 text-xs text-slate-400">
                Shelf count
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  value={dims.weighted_shelf_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, weighted_shelf_count: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Min gap (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0.01}
                  value={dims.weighted_min_gap_mm}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 0.01) {
                      return;
                    }
                    setDims((d) => ({ ...d, weighted_min_gap_mm: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[6rem] flex-col gap-1 text-xs text-slate-400">
                W lower
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0.01}
                  step={0.1}
                  value={dims.weighted_band_lower}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v <= 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, weighted_band_lower: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[6rem] flex-col gap-1 text-xs text-slate-400">
                W mid
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0.01}
                  step={0.1}
                  value={dims.weighted_band_middle}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v <= 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, weighted_band_middle: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[6rem] flex-col gap-1 text-xs text-slate-400">
                W upper
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={0.01}
                  step={0.1}
                  value={dims.weighted_band_upper}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v <= 0) {
                      return;
                    }
                    setDims((d) => ({ ...d, weighted_band_upper: v }));
                  }}
                />
              </label>
            </>
          )}
          {(dims.interiorMode === "equal_vertical_bays" || dims.interiorMode === "grid_uprights_explicit") && (
            <label className="flex min-w-[9rem] flex-col gap-1 text-xs text-slate-400">
              Upright thickness (mm)
              <input
                className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                type="text"
                placeholder="default 18"
                value={dims.upright_thickness_mm_str}
                onChange={(e) => setDims((d) => ({ ...d, upright_thickness_mm_str: e.target.value }))}
              />
            </label>
          )}
          {dims.interiorMode === "equal_vertical_bays" && (
            <>
              <label className="flex min-w-[7rem] flex-col gap-1 text-xs text-slate-400">
                Bay count
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  value={dims.bay_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, bay_count: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1 text-xs text-slate-400">
                Shelves per bay
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  value={dims.bay_shelf_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, bay_shelf_count: v }));
                  }}
                />
              </label>
            </>
          )}
          {dims.interiorMode === "grid_uprights_explicit" && (
            <>
              <label className="flex min-w-[7rem] flex-col gap-1 text-xs text-slate-400">
                Bay count
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="number"
                  min={1}
                  max={500}
                  value={dims.grid_bay_count}
                  onChange={(e) => {
                    const v = e.target.valueAsNumber;
                    if (!Number.isFinite(v) || v < 1 || v > 500) {
                      return;
                    }
                    setDims((d) => ({ ...d, grid_bay_count: v }));
                  }}
                />
              </label>
              <label className="flex min-w-[14rem] max-w-md flex-col gap-1 text-xs text-slate-400">
                Global shelf bottom Y (mm)
                <textarea
                  className="min-h-[5rem] rounded border border-slate-600 bg-slate-900 px-2 py-1.5 font-mono text-sm text-slate-100"
                  spellCheck={false}
                  value={dims.grid_shelf_y_str}
                  onChange={(e) => setDims((d) => ({ ...d, grid_shelf_y_str: e.target.value }))}
                />
              </label>
            </>
          )}
          <details className="flex w-full min-w-full flex-col gap-2 rounded border border-slate-700/60 bg-slate-950/20 px-2 py-2 text-xs text-slate-400">
            <summary className="cursor-pointer text-slate-300">Clearance / thickness (optional)</summary>
            <div className="flex flex-wrap items-end justify-center gap-3 pt-2">
              <label className="flex min-w-[10rem] flex-col gap-1">
                Panel thickness (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="text"
                  placeholder="0"
                  value={dims.clearance_panel_thickness_str}
                  onChange={(e) => setDims((d) => ({ ...d, clearance_panel_thickness_str: e.target.value }))}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1">
                Side inset (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="text"
                  placeholder="0"
                  value={dims.clearance_side_inset_str}
                  onChange={(e) => setDims((d) => ({ ...d, clearance_side_inset_str: e.target.value }))}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1">
                Front setback (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="text"
                  placeholder="0"
                  value={dims.clearance_front_setback_str}
                  onChange={(e) => setDims((d) => ({ ...d, clearance_front_setback_str: e.target.value }))}
                />
              </label>
              <label className="flex min-w-[8rem] flex-col gap-1">
                Shelf nosing (mm)
                <input
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="text"
                  placeholder="0"
                  value={dims.clearance_nosing_str}
                  onChange={(e) => setDims((d) => ({ ...d, clearance_nosing_str: e.target.value }))}
                />
              </label>
            </div>
          </details>
          {dims.interiorMode === "equal_spacing" && (
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
          )}
          {dims.interiorMode === "explicit_heights" && (
            <>
              <label className="flex min-w-[14rem] max-w-md flex-col gap-1 text-xs text-slate-400">
                Shelf bottom Y (mm), ascending
                <textarea
                  data-testid="textarea-explicit-shelf-y-mm"
                  className="min-h-[5.5rem] rounded border border-slate-600 bg-slate-900 px-2 py-1.5 font-mono text-sm text-slate-100"
                  spellCheck={false}
                  value={dims.explicit_shelf_bottoms_y_str}
                  onChange={(e) => {
                    setDims((d) => ({ ...d, explicit_shelf_bottoms_y_str: e.target.value }));
                  }}
                />
              </label>
              <label className="flex min-w-[9rem] flex-col gap-1 text-xs text-slate-400">
                Min gap between shelves (mm, optional)
                <input
                  data-testid="input-explicit-min-gap-mm"
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1.5 text-sm text-slate-100"
                  type="text"
                  inputMode="decimal"
                  placeholder="default 0"
                  value={dims.explicit_min_gap_mm_str}
                  onChange={(e) => {
                    setDims((d) => ({ ...d, explicit_min_gap_mm_str: e.target.value }));
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
