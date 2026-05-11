# WardrobeSpec JSON contract v1

- **Status**: accepted
- **Date**: 2026-05-11
- **Context**: The web UI and WASM need a single, versioned JSON document for parametric input; Rust (`serde`) and TypeScript (`zod`) must agree on shape and mm units.
- **Decision**: Introduce `WardrobeSpec` with integer `version` (currently `1`), internally tagged `layout` with `type: "straight_run"` and `width_mm` / `height_mm` / `depth_mm`, optional `extensions` object for forward-compatible metadata. Golden files live in `spec-fixtures/`; `furnigen-core` parses and validates; `furnigen-wasm` exposes `validateWardrobeSpecJson` and `normalizeWardrobeSpecJson`.
- **Alternatives considered**: Flattened fields without `layout` (rejected: harder to add L/U without breaking); `discriminatedUnion` in Zod with a single member (rejected: TS requires two variants—use straight-run schema until a second variant exists).
- **Consequences**: Any schema change updates golden fixtures, Rust, Zod, and this ADR or a superseding entry. Vitest does not load WASM (fetch/jsdom); WASM JSON tests run in `furnigen-wasm` `cargo test`.
- **Links**: Plan phase `spec-contract`.
