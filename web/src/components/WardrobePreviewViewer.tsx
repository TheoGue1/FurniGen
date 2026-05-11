import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

import { bufferGeometryFromPreviewMesh } from "../lib/preview/buffer-geometry-from-preview-mesh";
import { parsePreviewMeshJson } from "../lib/preview/preview-mesh";

type Props = {
  /** JSON string from `buildWardrobePreviewMeshJson` (WASM only). */
  meshJson: string | null;
};

type ViewerState = "idle" | "running" | "no-gl" | "error";

export function WardrobePreviewViewer({ meshJson }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);
  const [state, setState] = useState<ViewerState>("idle");
  const [detail, setDetail] = useState<string>("");

  useEffect(() => {
    const host = hostRef.current;
    if (!host || !meshJson) {
      setState("idle");
      setDetail("");
      return;
    }

    let cancelled = false;
    let rafId = 0;
    let renderer: THREE.WebGLRenderer | null = null;
    let controls: OrbitControls | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let geometry: THREE.BufferGeometry | null = null;
    let material: THREE.MeshStandardMaterial | null = null;

    setState("running");
    setDetail("");

    try {
      const meshData = parsePreviewMeshJson(meshJson);
      geometry = bufferGeometryFromPreviewMesh(meshData);
      geometry.computeBoundingBox();
      const box = geometry.boundingBox;
      if (!box) {
        throw new Error("missing bounding box");
      }
      const center = new THREE.Vector3();
      const size = new THREE.Vector3();
      box.getCenter(center);
      box.getSize(size);
      const maxDim = Math.max(size.x, size.y, size.z, 1);

      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
      renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
      renderer.domElement.setAttribute("data-testid", "preview-canvas");
      renderer.domElement.className = "h-full w-full touch-none outline-none";

      const gl = renderer.getContext() as WebGLRenderingContext | null;
      if (!gl) {
        renderer.dispose();
        renderer = null;
        geometry.dispose();
        geometry = null;
        if (!cancelled) {
          setState("no-gl");
          setDetail("WebGL context unavailable");
        }
        return;
      }

      const scene = new THREE.Scene();
      scene.background = new THREE.Color(0x0f172a);

      material = new THREE.MeshStandardMaterial({
        color: 0x94a3b8,
        metalness: 0.08,
        roughness: 0.82,
        side: THREE.DoubleSide,
      });
      const carcass = new THREE.Mesh(geometry, material);
      scene.add(carcass);

      const hemi = new THREE.HemisphereLight(0xffffff, 0x1e293b, 0.9);
      scene.add(hemi);
      const dir = new THREE.DirectionalLight(0xffffff, 0.55);
      dir.position.set(size.x * 1.4, size.y * 1.8, size.z * 2.8);
      scene.add(dir);

      const gridSize = Math.max(size.x, size.z) * 2.5;
      const grid = new THREE.GridHelper(gridSize, 48, 0x475569, 0x1e293b);
      grid.position.set(center.x, box.min.y, center.z);
      scene.add(grid);

      const axes = new THREE.AxesHelper(Math.min(maxDim * 0.22, 800));
      axes.position.copy(box.min);
      scene.add(axes);

      const camera = new THREE.PerspectiveCamera(42, 1, 1, maxDim * 50);
      camera.position.set(center.x + size.x * 1.15, center.y + size.y * 0.55, center.z + size.z * 1.85);
      camera.near = Math.max(1, maxDim / 2000);
      camera.far = maxDim * 80;
      camera.updateProjectionMatrix();
      camera.lookAt(center);

      controls = new OrbitControls(camera, renderer.domElement);
      controls.target.copy(center);
      controls.enableDamping = true;
      controls.dampingFactor = 0.06;
      controls.update();

      host.appendChild(renderer.domElement);

      const resize = () => {
        if (!renderer || !host) {
          return;
        }
        const { width, height } = host.getBoundingClientRect();
        const w = Math.max(1, Math.floor(width));
        const h = Math.max(1, Math.floor(height));
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        renderer.setSize(w, h, false);
      };

      resizeObserver = new ResizeObserver(resize);
      resizeObserver.observe(host);
      resize();

      const animate = () => {
        if (cancelled || !renderer || !controls) {
          return;
        }
        rafId = window.requestAnimationFrame(animate);
        controls.update();
        renderer.render(scene, camera);
      };
      animate();

      return () => {
        cancelled = true;
        window.cancelAnimationFrame(rafId);
        resizeObserver?.disconnect();
        resizeObserver = null;
        controls?.dispose();
        controls = null;
        if (renderer) {
          if (renderer.domElement.parentNode === host) {
            host.removeChild(renderer.domElement);
          }
          renderer.dispose();
          renderer = null;
        }
        geometry?.dispose();
        geometry = null;
        material?.dispose();
        material = null;
      };
    } catch (e) {
      geometry?.dispose();
      geometry = null;
      material?.dispose();
      material = null;
      renderer?.dispose();
      renderer = null;
      if (!cancelled) {
        setState("error");
        setDetail(e instanceof Error ? e.message : String(e));
      }
      return;
    }
  }, [meshJson]);

  return (
    <div
      className="relative h-[min(55vh,520px)] w-full max-w-4xl overflow-hidden rounded-lg border border-slate-700/80 bg-slate-950/40"
      data-testid="preview-viewer-host"
      aria-label="Wardrobe preview (millimeters, WASM mesh)"
    >
      {/* Three.js appends here only — keep React children out of this node */}
      <div ref={hostRef} className="absolute inset-0" />
      {state === "idle" && (
        <div
          className="absolute inset-0 z-10 flex items-center justify-center text-sm text-slate-500"
          data-testid="preview-idle"
        >
          Preview loads after WASM mesh is ready.
        </div>
      )}
      {state === "no-gl" && (
        <div
          className="absolute inset-0 z-10 flex items-center justify-center px-4 text-center text-sm text-amber-200/90"
          data-testid="preview-gl-fallback"
        >
          {detail || "WebGL unavailable in this environment."}
        </div>
      )}
      {state === "error" && (
        <div
          className="absolute inset-0 z-10 flex items-center justify-center px-4 text-center text-sm text-red-300/90"
          data-testid="preview-error"
        >
          {detail}
        </div>
      )}
    </div>
  );
}
