# Interior modes 7–11: random shelves, bays, grid, clearance

- **Status**: accepted
- **Date**: 2026-05-12
- **Context**: Extend `InteriorSpec` with procedural modes (seeded and weighted random placement), upright bays and aligned grid rows, and optional carcass/shelf clearance so preview and shelf geometry share one inner volume model.
- **Decision**:
  - **PRNG**: `SplitMix64` (Vigna, 2013) in `furnigen-core`; `seeded_random_min_gap_shelves` splits slack across `shelf_count + 1` segments with a uniform Dirichlet draw (independent `Exp(1)` spacings). `weighted_random_band_shelves` uses the same slack total but each slot draws `Gamma(α,1)` with integer `α = max(1, round(100 * band_weight))` where the slot’s vertical third (lower/middle/upper of **inner height**) picks the weight.
  - **Bays / grid**: `equal_vertical_bays_equal_spacing_shelves` repeats the same equal-vertical stack per bay with `bay_count − 1` uprights of optional `upright_thickness_mm` (default 18 mm). `grid_uprights_explicit_rows_shelves` shares one explicit bottom-Y list across all bays (aligned rows). Uprights are thin ±X preview slabs; BOM adds `upright_NN` with depth × inner height as nominal blank.
  - **Clearance**: Optional `clearance` on `WardrobeSpec` (`ClearanceSpec` in `inner_volume.rs`): `carcass_panel_thickness_mm` shrinks inner width/height by `2t` and depth by `t` (back only, front open). `side_inset_mm`, `front_setback_mm`, `shelf_nosing_mm` adjust shelf rectangles only. Preview shell uses **inner** dimensions; carcass BOM panels remain **outer** cut sizes (unchanged from pre-clearance behavior).
  - **Shelf part ids**: Sequential `shelf_NNN` (three digits) for arbitrary counts; replaces older `shelf_01` style in JSON/BOM/SVG tests.
- **Alternatives considered**: Nested `InteriorSpec` per bay (richer, heavier JSON) — deferred. Full carcass panel shrink to match inner box — deferred to keep existing BOM semantics.
- **Consequences**: Clients must accept new `interior` variants and optional `clearance`; fixture and WASM tests updated. Older tooling that hard-coded `shelf_01` ids should migrate to `shelf_001` or match by label/order.
- **Links**: Plan “Procedural interior” roadmap; prior ADR [0004](0004-wardrobe-spec-json-v1.md).
