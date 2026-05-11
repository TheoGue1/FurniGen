//! Triangle mesh for a minimal straight-run **open-front** carcass preview (mm → same numeric units in JSON).

use serde::{Deserialize, Serialize};

use crate::{LayoutSpec, WardrobeSpec};

/// Interleaved `x,y,z` positions and triangle `indices` (u32 element buffer).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreviewMesh {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
}

/// Builds an indexed mesh for the current spec layout. v1: straight run only (five panels, no door).
pub fn build_preview_mesh(spec: &WardrobeSpec) -> PreviewMesh {
    match &spec.layout {
        LayoutSpec::StraightRun {
            width_mm: w,
            height_mm: h,
            depth_mm: d,
        } => build_open_front_box_mm(*w as f32, *h as f32, *d as f32),
    }
}

fn push_vertex(buf: &mut Vec<f32>, x: f32, y: f32, z: f32) {
    buf.extend_from_slice(&[x, y, z]);
}

/// Two triangles; `corners` are CCW on the face when viewed from outside the solid.
fn push_quad(positions: &mut Vec<f32>, indices: &mut Vec<u32>, corners: [(f32, f32, f32); 4]) {
    let base = (positions.len() / 3) as u32;
    for (x, y, z) in corners {
        push_vertex(positions, x, y, z);
    }
    indices.extend_from_slice(&[
        base,
        base + 1,
        base + 2,
        base,
        base + 2,
        base + 3,
    ]);
}

/// Origin: bottom-left-back interior corner. +X width, +Y up, +Z toward room (front open at z = depth).
fn build_open_front_box_mm(w: f32, h: f32, d: f32) -> PreviewMesh {
    let mut positions = Vec::with_capacity(60);
    let mut indices = Vec::with_capacity(30);

    // Back (z = 0), outward -Z
    push_quad(
        &mut positions,
        &mut indices,
        [(0.0, 0.0, 0.0), (0.0, h, 0.0), (w, h, 0.0), (w, 0.0, 0.0)],
    );
    // Left (x = 0), outward -X
    push_quad(
        &mut positions,
        &mut indices,
        [(0.0, 0.0, 0.0), (0.0, 0.0, d), (0.0, h, d), (0.0, h, 0.0)],
    );
    // Right (x = w), outward +X
    push_quad(
        &mut positions,
        &mut indices,
        [(w, 0.0, 0.0), (w, h, 0.0), (w, h, d), (w, 0.0, d)],
    );
    // Bottom (y = 0), outward -Y
    push_quad(
        &mut positions,
        &mut indices,
        [(0.0, 0.0, 0.0), (w, 0.0, 0.0), (w, 0.0, d), (0.0, 0.0, d)],
    );
    // Top (y = h), outward +Y
    push_quad(
        &mut positions,
        &mut indices,
        [(0.0, h, 0.0), (0.0, h, d), (w, h, d), (w, h, 0.0)],
    );

    PreviewMesh {
        positions,
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_wardrobe_spec_json;

    #[test]
    fn straight_run_mesh_vertex_and_index_counts() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let mesh = build_preview_mesh(&spec);
        assert_eq!(mesh.positions.len(), 60); // 5 * 4 verts * 3
        assert_eq!(mesh.indices.len(), 30); // 5 * 2 tris * 3
    }

    #[test]
    fn straight_run_mesh_bbox_matches_spec() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let mesh = build_preview_mesh(&spec);
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for chunk in mesh.positions.chunks_exact(3) {
            let (x, y, z) = (chunk[0], chunk[1], chunk[2]);
            min[0] = min[0].min(x);
            min[1] = min[1].min(y);
            min[2] = min[2].min(z);
            max[0] = max[0].max(x);
            max[1] = max[1].max(y);
            max[2] = max[2].max(z);
        }
        assert!((min[0] - 0.0).abs() < 1e-3);
        assert!((min[1] - 0.0).abs() < 1e-3);
        assert!((min[2] - 0.0).abs() < 1e-3);
        assert!((max[0] - 2400.0).abs() < 1e-3);
        assert!((max[1] - 2200.0).abs() < 1e-3);
        assert!((max[2] - 600.0).abs() < 1e-3);
    }
}
