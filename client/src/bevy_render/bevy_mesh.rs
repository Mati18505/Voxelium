use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
    transform::components::Transform,
};

use crate::chunk_mesh_builder::{ChunkMesh, MaterialName};

#[derive(Debug, Default, Clone)]
pub struct BevyChunkMesh {
    pub layers: HashMap<MaterialName, Mesh>,
    pub transform: Transform,
}

impl BevyChunkMesh {
    pub fn apply_transform(&mut self, transform: Transform) {
        self.transform = self.transform * transform;
    }
}

impl From<ChunkMesh> for BevyChunkMesh {
    fn from(chunk_mesh: ChunkMesh) -> Self {
        let mut bevy_mesh = BevyChunkMesh::default();

        for (material_name, layer) in chunk_mesh.layers {
            if layer.vertex_index == 0 {
                continue;
            }

            let normals: Vec<[f32; 3]> = layer
                .normals
                .iter()
                .map(|e| [e[0] as f32, e[1] as f32, e[2] as f32])
                .collect();
            let triangles: Vec<u32> = layer.triangles.iter().map(|e| *e as u32).collect();
            let texture_indexes: Vec<[f32; 2]> = layer
                .texture_indexes
                .iter()
                .map(|e| [*e as f32, 0.0])
                .collect();

            let mesh: Mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, layer.vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, layer.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, texture_indexes)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(Indices::U32(triangles));

            bevy_mesh.layers.insert(material_name, mesh);
        }

        bevy_mesh
    }
}
