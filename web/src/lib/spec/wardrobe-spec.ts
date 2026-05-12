import { z } from "zod";

/** Mirrors `furnigen_core::WARDROBE_SPEC_VERSION`. */
export const WARDROBE_SPEC_VERSION = 1 as const;

const straightRunLayoutSchema = z.object({
  type: z.literal("straight_run"),
  width_mm: z.number().finite().positive(),
  height_mm: z.number().finite().positive(),
  depth_mm: z.number().finite().positive(),
});

/** v1 straight run only; widen to `z.discriminatedUnion` when multiple `type` values exist. */
export const layoutSpecSchema = straightRunLayoutSchema;

/** Mirrors `furnigen_core::InteriorSpec`. */
export const interiorSpecSchema = z.discriminatedUnion("type", [
  z.object({ type: z.literal("stub") }),
  z.object({
    type: z.literal("equal_spacing_shelves"),
    shelf_count: z.number().int().min(1).max(500),
    shelf_thickness_mm: z.number().finite().positive().optional(),
  }),
]);

export const wardrobeSpecSchema = z.object({
  version: z.literal(WARDROBE_SPEC_VERSION),
  layout: layoutSpecSchema,
  interior: interiorSpecSchema.optional(),
  extensions: z.record(z.string(), z.unknown()).optional().default({}),
});

export type LayoutSpec = z.infer<typeof layoutSpecSchema>;
export type InteriorSpec = z.infer<typeof interiorSpecSchema>;
export type WardrobeSpec = z.infer<typeof wardrobeSpecSchema>;

export function parseWardrobeSpecJson(json: string): WardrobeSpec {
  const raw: unknown = JSON.parse(json);
  return wardrobeSpecSchema.parse(raw);
}
