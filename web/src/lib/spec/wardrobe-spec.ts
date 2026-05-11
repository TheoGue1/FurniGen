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

export const wardrobeSpecSchema = z.object({
  version: z.literal(WARDROBE_SPEC_VERSION),
  layout: layoutSpecSchema,
  extensions: z.record(z.string(), z.unknown()).optional().default({}),
});

export type LayoutSpec = z.infer<typeof layoutSpecSchema>;
export type WardrobeSpec = z.infer<typeof wardrobeSpecSchema>;

export function parseWardrobeSpecJson(json: string): WardrobeSpec {
  const raw: unknown = JSON.parse(json);
  return wardrobeSpecSchema.parse(raw);
}
