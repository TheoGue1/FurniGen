import { z } from "zod";

/** Parsed WASM `buildWardrobePreviewMeshJson` payload (mm units, same numeric scale as Three). */
export const previewMeshSchema = z
  .object({
    positions: z.array(z.number().finite()),
    indices: z.array(z.number().int().nonnegative()),
  })
  .superRefine((data, ctx) => {
    if (data.positions.length % 3 !== 0) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: "positions length must be a multiple of 3 (xyz triples)",
      });
    }
    const nVerts = data.positions.length / 3;
    for (let i = 0; i < data.indices.length; i++) {
      const idx = data.indices[i];
      if (idx !== undefined && idx >= nVerts) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: `index ${idx} out of range for ${nVerts} vertices`,
        });
        return;
      }
    }
  });

export type PreviewMesh = z.infer<typeof previewMeshSchema>;

export function parsePreviewMeshJson(json: string): PreviewMesh {
  const raw: unknown = JSON.parse(json);
  return previewMeshSchema.parse(raw);
}
