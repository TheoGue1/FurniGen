# Export roadmap: BOM, 2D, then mesh

- **Status**: accepted
- **Date**: 2026-05-11
- **Context**: After interactive preview, the product targets cut list/BOM, 2D panel outlines (SVG/DXF), and 3D mesh export (glTF/OBJ). Implementation spans `furnigen-core` and `furnigen-wasm`; sequencing affects what downstream features can assume stable.
- **Decision**: Ship exports in this order: **(1) BOM / cut list**, **(2) 2D panel outlines**, **(3) 3D mesh (OBJ + glTF)**. BOM and 2D both derive from the same **panel blank list** (nominal rectangles in mm). Mesh exports reuse the existing **preview indexed mesh** so preview and export stay aligned until a dedicated “manufacturing mesh” exists.
- **Alternatives considered**: Mesh-first (rejected: panel dimensions for shop drawings would be duplicated or inferred from mesh). Parallel-only with no order (rejected: harder to document dependencies; BOM still logically precedes kerf-aware 2D in later milestones).
- **Consequences**: v1 BOM omits carcass `thickness_mm` (not in `WardrobeSpec` yet). 2D is **nominal outer rectangles** only—no kerf or thickness offsets. glTF uses a single embedded `data:` buffer for portability in the browser. Follow-up ADRs can add thickness to the spec, DXF tool paths, and manufacturing-tolerant mesh when those milestones land.
- **Links**: [`.cursor/rules/furnigen-project-context.mdc`](../../.cursor/rules/furnigen-project-context.mdc)
