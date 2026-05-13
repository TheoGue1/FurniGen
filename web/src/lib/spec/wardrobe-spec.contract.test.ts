/** Golden JSON contract: Zod here; WASM JSON entrypoints are covered in `furnigen-wasm` `cargo test` (jsdom cannot fetch the `.wasm` bundle). */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { parseWardrobeSpecJson, wardrobeSpecSchema } from "./wardrobe-spec";

const here = dirname(fileURLToPath(import.meta.url));
const fixture = (name: string) =>
  readFileSync(join(here, "..", "..", "..", "..", "spec-fixtures", name), "utf8");

describe("WardrobeSpec Zod contract (golden fixtures)", () => {
  it("accepts v1 minimal golden", () => {
    const raw = fixture("wardrobe-spec-v1-minimal.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.version).toBe(1);
    expect(spec.layout.type).toBe("straight_run");
    if (spec.layout.type === "straight_run") {
      expect(spec.layout.width_mm).toBe(2400);
      expect(spec.layout.height_mm).toBe(2200);
      expect(spec.layout.depth_mm).toBe(600);
    }
    expect(spec.extensions).toEqual({});
    expect(spec.interior).toBeUndefined();
  });

  it("accepts golden equal_spacing_shelves fixture", () => {
    const raw = fixture("wardrobe-spec-v1-equal-spacing-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "equal_spacing_shelves",
      shelf_count: 4,
    });
  });

  it("rejects explicit_shelf_heights with negative min_gap_mm", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 2400, height_mm: 2200, depth_mm: 600 },
      interior: {
        type: "explicit_shelf_heights",
        shelf_bottom_y_mm: [400, 1000],
        min_gap_mm: -1,
      },
    });
    expect(() => parseWardrobeSpecJson(raw)).toThrow();
  });

  it("rejects equal_spacing_shelves with shelf_count below 1", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 1000, height_mm: 2200, depth_mm: 600 },
      interior: { type: "equal_spacing_shelves", shelf_count: 0 },
    });
    expect(() => parseWardrobeSpecJson(raw)).toThrow();
  });

  it("accepts golden zones_equal_fill_shelves fixture", () => {
    const raw = fixture("wardrobe-spec-v1-zones-equal-fill-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "zones_equal_fill_shelves",
      bottom_zone_mm: 500,
      top_reserve_mm: 300,
      shelf_count: 3,
    });
  });

  it("accepts golden golden_ratio_ladder fixture", () => {
    const raw = fixture("wardrobe-spec-v1-golden-ratio-ladder-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "golden_ratio_ladder_shelves",
      rungs: 3,
    });
  });

  it("accepts golden two_tier_rhythm_shelves fixture", () => {
    const raw = fixture("wardrobe-spec-v1-two-tier-rhythm-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "two_tier_rhythm_shelves",
      transition_y_mm: 900,
      gap_lower_mm: 80,
      gap_upper_mm: 200,
    });
  });

  it("accepts golden max_shelves_min_segment_shelves fixture", () => {
    const raw = fixture("wardrobe-spec-v1-max-shelves-min-segment-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "max_shelves_min_segment_shelves",
      min_vertical_segment_mm: 100,
    });
  });

  it("rejects golden_ratio_ladder_shelves with rungs below 1", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 2400, height_mm: 2200, depth_mm: 600 },
      interior: { type: "golden_ratio_ladder_shelves", rungs: 0 },
    });
    expect(() => parseWardrobeSpecJson(raw)).toThrow();
  });

  it("rejects zones_equal_fill_shelves with negative bottom_zone_mm", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 2400, height_mm: 2200, depth_mm: 600 },
      interior: {
        type: "zones_equal_fill_shelves",
        bottom_zone_mm: -1,
        top_reserve_mm: 0,
        shelf_count: 1,
      },
    });
    expect(() => parseWardrobeSpecJson(raw)).toThrow();
  });

  it("accepts golden explicit_shelf_heights fixture", () => {
    const raw = fixture("wardrobe-spec-v1-explicit-shelf-heights.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "explicit_shelf_heights",
      shelf_bottom_y_mm: [400, 1000, 1600],
    });
  });

  it("accepts explicit_shelf_heights with optional fields", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 2400, height_mm: 2200, depth_mm: 600 },
      interior: {
        type: "explicit_shelf_heights",
        shelf_bottom_y_mm: [400, 1000, 1600],
        shelf_thickness_mm: 25,
        min_gap_mm: 2,
      },
    });
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({
      type: "explicit_shelf_heights",
      shelf_bottom_y_mm: [400, 1000, 1600],
      shelf_thickness_mm: 25,
      min_gap_mm: 2,
    });
  });

  it("accepts equal_spacing_shelves interior", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 2400, height_mm: 2200, depth_mm: 600 },
      interior: { type: "equal_spacing_shelves", shelf_count: 3 },
    });
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({ type: "equal_spacing_shelves", shelf_count: 3 });
  });

  it("accepts optional interior stub and round-trips", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 1, height_mm: 2, depth_mm: 3 },
      interior: { type: "stub" },
    });
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior).toEqual({ type: "stub" });
    const again = wardrobeSpecSchema.parse(JSON.parse(JSON.stringify(spec)));
    expect(again).toEqual(spec);
  });

  it("rejects invalid depth from golden", () => {
    const raw = fixture("wardrobe-spec-v1-invalid-depth.json");
    expect(() => parseWardrobeSpecJson(raw)).toThrow();
  });

  it("accepts golden with extensions and round-trips through JSON", () => {
    const raw = fixture("wardrobe-spec-v1-with-extensions.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.extensions.reserved).toBe(true);
    const encoded = JSON.stringify(spec);
    const again = wardrobeSpecSchema.parse(JSON.parse(encoded));
    expect(again).toEqual(spec);
  });

  it("accepts golden seeded_random_min_gap fixture", () => {
    const raw = fixture("wardrobe-spec-v1-seeded-random-min-gap-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior?.type).toBe("seeded_random_min_gap_shelves");
  });

  it("accepts golden weighted_random_band fixture", () => {
    const raw = fixture("wardrobe-spec-v1-weighted-random-band-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior?.type).toBe("weighted_random_band_shelves");
  });

  it("accepts golden equal_vertical_bays fixture", () => {
    const raw = fixture("wardrobe-spec-v1-equal-vertical-bays-shelves.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior?.type).toBe("equal_vertical_bays_equal_spacing_shelves");
  });

  it("accepts golden grid_uprights_explicit_rows fixture", () => {
    const raw = fixture("wardrobe-spec-v1-grid-uprights-explicit-rows.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.interior?.type).toBe("grid_uprights_explicit_rows_shelves");
  });

  it("accepts golden clearance fixture", () => {
    const raw = fixture("wardrobe-spec-v1-clearance-thickness.json");
    const spec = parseWardrobeSpecJson(raw);
    expect(spec.clearance?.carcass_panel_thickness_mm).toBe(18);
  });

  it("rejects weighted_random_band_shelves with non-positive band weight", () => {
    const raw = JSON.stringify({
      version: 1,
      layout: { type: "straight_run", width_mm: 2400, height_mm: 2200, depth_mm: 600 },
      interior: {
        type: "weighted_random_band_shelves",
        seed: 1,
        shelf_count: 2,
        min_gap_mm: 50,
        band_weight_lower: 0,
        band_weight_middle: 1,
        band_weight_upper: 1,
      },
    });
    expect(() => parseWardrobeSpecJson(raw)).toThrow();
  });
});
