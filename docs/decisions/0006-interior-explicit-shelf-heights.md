# Interior mode: explicit shelf bottom Y list

- **Status**: accepted
- **Date**: 2026-05-12
- **Context**: Users need fixed shelf elevations (mm from inner floor) instead of algorithmic equal spacing, with strict validation for BOM and preview.
- **Decision**: Add `InteriorSpec::ExplicitShelfHeights` with `shelf_bottom_y_mm: Vec<f64>` (strictly ascending), optional `shelf_thickness_mm` and `min_gap_mm` (default gap 0 between boards when omitted). Validation and shelf quads/panel rows reuse the same shelf geometry path as equal spacing once bottoms are resolved.
- **Alternatives considered**: Accept unsorted lists and sort in core (rejected: hides user mistakes; UI may sort later as a separate affordance). Require `min_gap_mm` always (rejected: optional keeps simple flush stacks).
- **Consequences**: JSON remains `version: 1`; new `type` value `explicit_shelf_heights`. Empty `shelf_bottom_y_mm` is valid (no shelf boards). WASM entrypoints unchanged.
- **Links**: [0004-wardrobe-spec-json-v1.md](0004-wardrobe-spec-json-v1.md)
