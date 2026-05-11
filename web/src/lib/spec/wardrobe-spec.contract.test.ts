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
});
