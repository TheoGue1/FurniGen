import { describe, expect, it } from "vitest";

import { bufferGeometryFromPreviewMesh } from "./buffer-geometry-from-preview-mesh";
import { parsePreviewMeshJson } from "./preview-mesh";

describe("parsePreviewMeshJson", () => {
  it("accepts a minimal WASM-shaped payload", () => {
    const raw = JSON.stringify({
      positions: [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0],
      indices: [0, 1, 2, 0, 2, 3],
    });
    const mesh = parsePreviewMeshJson(raw);
    expect(mesh.positions).toHaveLength(12);
    expect(mesh.indices).toHaveLength(6);
  });

  it("rejects out-of-range indices", () => {
    const raw = JSON.stringify({
      positions: [0, 0, 0, 1, 0, 0, 1, 1, 0],
      indices: [0, 1, 9],
    });
    expect(() => parsePreviewMeshJson(raw)).toThrow();
  });
});

describe("bufferGeometryFromPreviewMesh", () => {
  it("produces indexed triangle geometry", () => {
    const mesh = parsePreviewMeshJson(
      JSON.stringify({
        positions: [0, 0, 0, 10, 0, 0, 10, 10, 0, 0, 10, 0],
        indices: [0, 1, 2, 0, 2, 3],
      })
    );
    const geom = bufferGeometryFromPreviewMesh(mesh);
    expect(geom.index?.count).toBe(6);
    geom.dispose();
  });
});
