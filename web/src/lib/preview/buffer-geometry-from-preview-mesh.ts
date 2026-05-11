import * as THREE from "three";

import type { PreviewMesh } from "./preview-mesh";

/** Builds a Three.js mesh from WASM/core preview data only (no TS-side box math). */
export function bufferGeometryFromPreviewMesh(mesh: PreviewMesh): THREE.BufferGeometry {
  const geom = new THREE.BufferGeometry();
  geom.setAttribute("position", new THREE.Float32BufferAttribute(mesh.positions, 3));
  geom.setIndex(mesh.indices);
  geom.computeVertexNormals();
  return geom;
}
