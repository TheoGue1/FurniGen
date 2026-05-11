//! Wavefront OBJ and glTF 2.0 (JSON + embedded buffer) from preview mesh buffers.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::json;
use thiserror::Error;

use crate::PreviewMesh;

#[derive(Debug, Error, PartialEq)]
pub enum MeshExportError {
    #[error("mesh has no vertices")]
    EmptyPositions,
    #[error("mesh index count must be a multiple of 3")]
    BadIndexCount,
}

/// Wavefront OBJ: mm coordinates, triangular faces, 1-based indices.
pub fn preview_mesh_to_obj(
    mesh: &PreviewMesh,
    object_name: &str,
) -> Result<String, MeshExportError> {
    if mesh.positions.is_empty() {
        return Err(MeshExportError::EmptyPositions);
    }
    if mesh.indices.len() % 3 != 0 {
        return Err(MeshExportError::BadIndexCount);
    }
    let mut out = String::from("# FurniGen — units: mm\n");
    out.push_str(&format!("o {object_name}\n"));
    for chunk in mesh.positions.chunks_exact(3) {
        out.push_str(&format!(
            "v {} {} {}\n",
            f64::from(chunk[0]),
            f64::from(chunk[1]),
            f64::from(chunk[2])
        ));
    }
    for tri in mesh.indices.chunks_exact(3) {
        out.push_str(&format!("f {} {} {}\n", tri[0] + 1, tri[1] + 1, tri[2] + 1));
    }
    Ok(out)
}

/// glTF 2.0 JSON with a single `buffers[0]` data-URI carrying interleaved f32 positions and u32 indices.
pub fn preview_mesh_to_gltf(mesh: &PreviewMesh) -> Result<String, MeshExportError> {
    if mesh.positions.is_empty() {
        return Err(MeshExportError::EmptyPositions);
    }
    if mesh.indices.len() % 3 != 0 {
        return Err(MeshExportError::BadIndexCount);
    }
    let vert_count = mesh.positions.len() / 3;
    let index_count = mesh.indices.len();

    let pos_bytes = mesh.positions.len() * 4;
    let mut bin: Vec<u8> = Vec::with_capacity(pos_bytes + mesh.indices.len() * 4);
    for p in &mesh.positions {
        bin.extend_from_slice(&p.to_le_bytes());
    }
    for i in &mesh.indices {
        bin.extend_from_slice(&i.to_le_bytes());
    }

    let (min, max) = bbox(mesh);

    let uri = format!("data:application/octet-stream;base64,{}", B64.encode(&bin));

    let doc = json!({
        "asset": { "version": "2.0", "generator": "FurniGen furnigen-core" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0, "name": "wardrobe_preview" }],
        "meshes": [{
            "primitives": [{
                "attributes": { "POSITION": 0 },
                "indices": 1,
                "mode": 4
            }]
        }],
        "accessors": [
            {
                "bufferView": 0,
                "byteOffset": 0,
                "componentType": 5126,
                "count": vert_count,
                "type": "VEC3",
                "min": [min[0] as f64, min[1] as f64, min[2] as f64],
                "max": [max[0] as f64, max[1] as f64, max[2] as f64]
            },
            {
                "bufferView": 1,
                "byteOffset": 0,
                "componentType": 5125,
                "count": index_count,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": pos_bytes,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": pos_bytes,
                "byteLength": index_count * 4,
                "target": 34963
            }
        ],
        "buffers": [{
            "uri": uri,
            "byteLength": bin.len()
        }]
    });

    // `serde_json::Value` always serializes to a string.
    Ok(serde_json::to_string(&doc).expect("gltf Value serialization"))
}

fn bbox(mesh: &PreviewMesh) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for c in mesh.positions.chunks_exact(3) {
        for i in 0..3 {
            min[i] = min[i].min(c[i]);
            max[i] = max[i].max(c[i]);
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_preview_mesh, parse_wardrobe_spec_json};

    #[test]
    fn obj_matches_vertex_and_face_counts() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let mesh = build_preview_mesh(&spec);
        let obj = preview_mesh_to_obj(&mesh, "wardrobe").unwrap();
        let v_lines = obj.lines().filter(|l| l.starts_with("v ")).count();
        let f_lines = obj.lines().filter(|l| l.starts_with('f')).count();
        assert_eq!(v_lines, mesh.positions.len() / 3);
        assert_eq!(f_lines, mesh.indices.len() / 3);
    }

    #[test]
    fn gltf_is_valid_json_with_buffer() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let mesh = build_preview_mesh(&spec);
        let gltf = preview_mesh_to_gltf(&mesh).unwrap();
        let v: serde_json::Value = serde_json::from_str(&gltf).unwrap();
        assert_eq!(v["asset"]["version"], "2.0");
        let uri = v["buffers"][0]["uri"].as_str().unwrap();
        assert!(uri.starts_with("data:application/octet-stream;base64,"));
    }
}
