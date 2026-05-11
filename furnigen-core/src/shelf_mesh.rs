//! Triangle helpers for horizontal shelf boards in preview meshes (mm → `f32` buffer units).

fn push_vertex(buf: &mut Vec<f32>, x: f32, y: f32, z: f32) {
    buf.extend_from_slice(&[x, y, z]);
}

/// Two triangles; `corners` are CCW on the face when viewed from outside the solid (same convention as `preview_mesh`).
fn push_quad(positions: &mut Vec<f32>, indices: &mut Vec<u32>, corners: [(f32, f32, f32); 4]) {
    let base = (positions.len() / 3) as u32;
    for (x, y, z) in corners {
        push_vertex(positions, x, y, z);
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Top face of a shelf board at `y_top_mm` (outward normal **+Y**), full span in X and Z.
pub fn append_shelf_top_face_mm(
    positions: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    y_top_mm: f32,
    width_mm: f32,
    depth_mm: f32,
) {
    let w = width_mm;
    let d = depth_mm;
    let y = y_top_mm;
    push_quad(
        positions,
        indices,
        [(0.0, y, 0.0), (0.0, y, d), (w, y, d), (w, y, 0.0)],
    );
}

/// Bottom face at `y_bottom_mm` (outward normal **−Y**).
pub fn append_shelf_bottom_face_mm(
    positions: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    y_bottom_mm: f32,
    width_mm: f32,
    depth_mm: f32,
) {
    let w = width_mm;
    let d = depth_mm;
    let y = y_bottom_mm;
    push_quad(
        positions,
        indices,
        [(0.0, y, 0.0), (w, y, 0.0), (w, y, d), (0.0, y, d)],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shelf_top_face_adds_quad() {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        append_shelf_top_face_mm(&mut positions, &mut indices, 1000.0, 2400.0, 600.0);
        assert_eq!(positions.len(), 12);
        assert_eq!(indices.len(), 6);
    }
}
