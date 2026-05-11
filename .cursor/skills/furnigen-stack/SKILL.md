---
name: furnigen-stack
description: >-
  FurniGen stack orientation (Rust core/WASM, React+Vite+Tailwind+Three, mm units).
  Use when bootstrapping crates, wiring wasm-pack, or tracing spec data from UI to core.
---

# FurniGen stack skill

## Quick map

| Area        | Location / role |
|------------|------------------|
| Domain + geometry | `furnigen-core` (Rust) |
| Browser FFI     | `furnigen-wasm` (`wasm-bindgen`, wasm-pack) |
| UI + viewer     | `web/` (React, TS, Vite, Tailwind, Three.js) |

## Invariants

- Spec and Rust use **mm**; JSON on the wire is mm; UI may show in/mm with explicit conversion.
- **Straight runs** first; future corners live behind schema extension—do not block v1 on L/U shapes.
- Three.js consumes **WASM/core outputs**, not duplicated parametric math in TS.

## When changing contracts

- Add a short ADR under [`docs/decisions/`](../../../docs/decisions/) for schema or export format changes; follow [`.cursor/rules/technical-decision-log.mdc`](../../../.cursor/rules/technical-decision-log.mdc) and update [`docs/decisions/README.md`](../../../docs/decisions/README.md).
- Keep **Zod** (TS) and **`serde`** (Rust) in lockstep; add golden JSON fixtures when touching the spec.

## References

- Project rules: `.cursor/rules/` (`furnigen-project-context`, `technical-decision-log`, `typescript-react-vite`, `rust-workspace`, etc.).
- Decision log: [`docs/decisions/`](../../../docs/decisions/).
- External inspiration list: [awesome-cursorrules](https://github.com/PatrickJS/awesome-cursorrules) (not domain-specific).
