# Interior: zones + equal-fill shelves

- **Status**: accepted
- **Date**: 2026-05-12
- **Context**: Users need hanging/long-item zones at the bottom and top of a run while still distributing a fixed number of shelf boards with equal air gaps in the remaining middle band.
- **Decision**: Add `InteriorSpec::ZonesEqualFillShelves` with `bottom_zone_mm`, `top_reserve_mm` (non-negative, sum strictly less than inner height), `shelf_count` (≥ 1), optional `shelf_thickness_mm`. Shelf bottom positions equal those from [`equal_spacing_shelf_bottoms_mm`](../../furnigen-core/src/interior_shelves.rs) applied to band height `inner_height - bottom_zone - top_reserve`, then shifted by `+ bottom_zone_mm` (same gap semantics as full-height equal spacing, confined to the band).
- **Alternatives considered**: Separate “margin” vs “clear height” fields — rejected to stay aligned with the procedural roadmap wording (`bottom_zone_mm` / `top_reserve_mm`).
- **Consequences**: Preview mesh, panel blanks, BOM, and 2D exports pick up shelf rows when this mode validates; UI gains another interior mode. Future carcass thickness / clearance (roadmap) will shrink effective inner height before placement.
- **Links**: Procedural interior roadmap; [`0006`](0006-interior-explicit-shelf-heights.md).
