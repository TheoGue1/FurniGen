# Interior: golden ratio ladder shelves

- **Status**: accepted
- **Date**: 2026-05-12
- **Context**: Procedural roadmap item (4): deterministic shelf positions from inner height using fixed φ ratios, without user height lists or RNG.
- **Decision**: Add `InteriorSpec::GoldenRatioLadderShelves` with `rungs` (≥ 1, number of horizontal shelf boards), optional `shelf_thickness_mm`. Air gaps under the first shelf, between boards, and above the last shelf are proportional to φ^0…φ^n with φ = (1+√5)/2 and n = `rungs`; total air equals `inner_height − rungs × thickness`. Shelf bottoms accumulate from the inner floor. Same preview/BOM/2D path as other shelf modes once bottoms resolve; validation fails if any gap would fall below the core minimum air gap.
- **Alternatives considered**: φ weights largest at the floor (reversed ladder) — not chosen; current weights grow upward so the top opening is largest. Recursive bisection at φ — rejected in favor of one closed-form weight sum for clarity and tests.
- **Consequences**: Zod + UI gain another interior mode; golden fixture documents a sample JSON. Future carcass thickness (roadmap) shrinks effective inner height before placement.
- **Links**: Procedural interior roadmap; [`0007`](0007-interior-zones-equal-fill-shelves.md).
