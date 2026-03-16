use std::iter;

use bevy::{asset::RenderAssetUsages, log::info_span, math::Vec3, mesh::{Indices, Mesh, MeshBuilder, PrimitiveTopology}};
use shared::entities::{BlockSide, Direction};

use crate::chunk_mesh_builder::meshers::{ChunkMeshData, Quad};

#[derive(Clone, Debug, Default)]
pub struct ChunkMeshBuilder {
    pub chunk_mesh_data: ChunkMeshData,
}

struct MeshData {
    positions: Vec<Vec3>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    /// Plane mesh builder without rotation, scale and subdivision.
    fn build_face(quad: &Quad) -> MeshData {
        let z_vertex_count = 2;
        let x_vertex_count = 2;
        let num_vertices = (z_vertex_count * x_vertex_count) as usize;
        let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

        let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let u = -0.5 + tx;
                let v = -0.5 + tz;

                let normal: Direction = quad.facing_side.into();
                let normal = Vec3 {
                    x: normal.x as f32,
                    y: normal.y as f32,
                    z: normal.z as f32,
                };
                let pos = Self::map_face(quad.facing_side, u, v);
                positions.push(pos);
                normals.push(normal.to_array());
                uvs.push([tx, tz]);
            }
        }

        for z in 0..z_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = z * x_vertex_count + x;
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
            }
        }

        MeshData { positions, normals, uvs, indices }
    }

    fn map_face(facing_side: BlockSide, u: f32, v: f32) -> Vec3 {
        match facing_side {
            BlockSide::Right => Vec3::new(0.0, v, u),
            BlockSide::Left => Vec3::new(0.0, v, -u),

            BlockSide::Top => Vec3::new(u, 0.0, v),
            BlockSide::Bottom => Vec3::new(u, 0.0, -v),

            BlockSide::Front => Vec3::new(-u, v, 0.0),
            BlockSide::Back => Vec3::new(u, v, 0.0),
        }
    }
}

impl MeshBuilder for ChunkMeshBuilder {
    fn build(&self) -> Mesh {
        let _ = info_span!(
            "chunk_mesh_builder",
            name = "chunk_mesh_builder"
        )
            .entered();

        let num_planes = self.chunk_mesh_data.quads.len();
        let num_vertices = num_planes * 4;
        let num_indices = num_planes * 6;

        let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);
        let mut uvs_2: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);

        for (i, quad) in self.chunk_mesh_data.quads.iter().enumerate() {
            let translation = Vec3{
                x: quad.block_pos.x as f32,
                y: quad.block_pos.y as f32,
                z: quad.block_pos.z as f32,
            }; 
            let face = Self::build_face(quad);

            let normal: Direction = quad.facing_side.into();
            let normal = Vec3 {
                x: normal.x as f32,
                y: normal.y as f32,
                z: normal.z as f32,
            };

            positions.extend(face.positions.iter().map(|pos| pos + normal * 0.5 + translation));
            normals.extend(face.normals);
            uvs.extend(face.uvs);
            indices.extend(face.indices.iter().map(|e| *e + 4*i as u32));
            uvs_2.extend(iter::repeat_n([quad.uv_2 as f32, 0.0], 4));
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, uvs_2)
    }
}
