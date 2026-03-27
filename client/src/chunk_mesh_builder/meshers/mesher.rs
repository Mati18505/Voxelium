use lazy_static::lazy_static;
use std::iter;

use bevy::{
    asset::RenderAssetUsages,
    log::info_span,
    math::{UVec3, Vec3},
    mesh::{Indices, Mesh, MeshBuilder, MeshVertexAttribute, PrimitiveTopology, VertexFormat},
};
use shared::entities::{BlockSide, Direction, CHUNK_SIZE};

use crate::chunk_mesh_builder::{ChunkMeshData, FaceData};

pub const ATTRIBUTE_BLOCK_IN_CHUNK_POS_INDEX: MeshVertexAttribute = MeshVertexAttribute::new(
    "block_in_chunk_pos_index",
    Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE,
    VertexFormat::Uint32,
);
pub const ATTRIBUTE_BLOCK_SIDE: MeshVertexAttribute = MeshVertexAttribute::new(
    "block_side",
    Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE + 1,
    VertexFormat::Uint32,
);
pub const ATTRIBUTE_UV: MeshVertexAttribute = MeshVertexAttribute::new(
    "uv",
    Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE + 2,
    VertexFormat::Uint32,
);
pub const ATTRIBUTE_STORAGE_INDEX: MeshVertexAttribute = MeshVertexAttribute::new(
    "storage_index",
    Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE + 3,
    VertexFormat::Uint32,
);

#[derive(Clone, Debug, Default)]
pub struct ChunkMeshBuilder {
    pub chunk_mesh_data: ChunkMeshData,
}

#[derive(Debug, Default)]
struct MeshData {
    positions: Vec<Vec3>,
    normals: Vec<Normal>,
    uvs: Vec<UV>,
    indices: Vec<u32>,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Normal {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Front = 4,
    Back = 5,
}

impl TryFrom<u32> for Normal {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Normal::Up),
            1 => Ok(Normal::Down),
            2 => Ok(Normal::Left),
            3 => Ok(Normal::Right),
            4 => Ok(Normal::Front),
            5 => Ok(Normal::Back),
            _ => Err(()),
        }
    }
}

impl From<BlockSide> for Normal {
    fn from(side: BlockSide) -> Self {
        match side {
            BlockSide::Front => Self::Front,
            BlockSide::Back => Self::Back,
            BlockSide::Right => Self::Right,
            BlockSide::Left => Self::Left,
            BlockSide::Top => Self::Up,
            BlockSide::Bottom => Self::Down,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UV {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl TryFrom<u32> for UV {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UV::TopLeft),
            1 => Ok(UV::TopRight),
            2 => Ok(UV::BottomLeft),
            3 => Ok(UV::BottomRight),
            _ => Err(()),
        }
    }
}

impl ChunkMeshBuilder {
    /// Plane mesh builder without rotation, scale and subdivision.
    fn build_face(facing_side: BlockSide) -> MeshData {
        let z_vertex_count = 2;
        let x_vertex_count = 2;
        let num_vertices = (z_vertex_count * x_vertex_count) as usize;
        let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

        let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<Normal> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<UV> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let u = -0.5 + tx;
                let v = -0.5 + tz;

                let uv_index = z * x_vertex_count + x;

                let pos = Self::map_face(facing_side, u, v);
                positions.push(pos);
                normals.push(Normal::from(facing_side));
                uvs.push(UV::try_from(uv_index).unwrap());
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

        MeshData {
            positions,
            normals,
            uvs,
            indices,
        }
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

    fn plane_pos_to_vertex_pos(plane_pos: &Vec3, quad: &FaceData) -> UVec3 {
        let block_pos = Vec3 {
            x: quad.block_pos.x as f32,
            y: quad.block_pos.y as f32,
            z: quad.block_pos.z as f32,
        };
        let normal: Direction = quad.facing_side.into();
        let normal = Vec3 {
            x: normal.x as f32,
            y: normal.y as f32,
            z: normal.z as f32,
        };

        let plane_offset = 0.5;
        let block_side_offset = normal * 0.5;
        let pos_offset = block_pos + block_side_offset + plane_offset;

        let pos = plane_pos + pos_offset;
        pos.floor().as_uvec3()
    }

    fn vertex_pos_to_index(pos: UVec3) -> u32 {
        // Legal range of vertex pos (chunks are connected).
        let size = CHUNK_SIZE as u32 + 1;

        pos.z * size * size + pos.y * size + pos.x
    }
}

lazy_static! {
    static ref FACES: [MeshData; 6] = {
        let mut faces: [MeshData; 6] = Default::default();

        for side in BlockSide::iterator().cloned() {
            faces[side as usize] = ChunkMeshBuilder::build_face(side);
        }

        faces
    };
}

impl MeshBuilder for ChunkMeshBuilder {
    fn build(&self) -> Mesh {
        let _ = info_span!("chunk_mesh_builder", name = "chunk_mesh_builder").entered();

        let num_planes = self.chunk_mesh_data.faces.len();
        let num_vertices = num_planes * 4;
        let num_indices = num_planes * 6;

        let mut position_idxs: Vec<u32> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<u32> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<u32> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);
        let mut storage_indices: Vec<u32> = Vec::with_capacity(num_vertices);

        for (i, quad) in self.chunk_mesh_data.faces.iter().enumerate() {
            let face = &FACES[quad.facing_side as usize];

            position_idxs.extend(
                face.positions
                    .iter()
                    .map(|plane_pos| Self::plane_pos_to_vertex_pos(plane_pos, quad))
                    .map(Self::vertex_pos_to_index),
            );
            normals.extend(face.normals.iter().map(|e| *e as u32));
            uvs.extend(face.uvs.iter().map(|e| *e as u32));

            let base_index = 4 * i as u32;
            indices.extend(face.indices.iter().map(|e| *e + base_index));
            storage_indices.extend(iter::repeat_n(quad.uv_2, 4));
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(ATTRIBUTE_BLOCK_IN_CHUNK_POS_INDEX, position_idxs)
        .with_inserted_attribute(ATTRIBUTE_BLOCK_SIDE, normals)
        .with_inserted_attribute(ATTRIBUTE_UV, uvs)
        .with_inserted_attribute(ATTRIBUTE_STORAGE_INDEX, storage_indices)
    }
}

#[cfg(test)]
mod tests {
    use shared::entities::BlockInChunkPos;

    use super::*;

    struct TransformInput {
        plane_vertex: Vec3,
        facing_side: BlockSide,
        block_pos: [usize; 3],
        expected: UVec3,
    }

    const TEST_CASES: [TransformInput; 3] = [
        TransformInput {
            plane_vertex: Vec3::splat(-0.5),
            facing_side: BlockSide::Left,
            block_pos: [0, 0, 0],
            expected: UVec3::ZERO,
        },
        TransformInput {
            plane_vertex: Vec3::splat(-0.5),
            facing_side: BlockSide::Top,
            block_pos: [15, 15, 15],
            expected: UVec3::splat(15),
        },
        TransformInput {
            plane_vertex: Vec3::splat(0.5),
            facing_side: BlockSide::Right,
            block_pos: [15, 15, 15],
            expected: UVec3::splat(16),
        },
    ];

    #[test]
    fn test_plane_pos_to_vertex_pos() {
        for case in TEST_CASES {
            let quad = FaceData {
                facing_side: case.facing_side,
                block_pos: BlockInChunkPos::new(
                    case.block_pos[0],
                    case.block_pos[1],
                    case.block_pos[2],
                ),
                uv_2: 0,
            };

            let transformed = ChunkMeshBuilder::plane_pos_to_vertex_pos(&case.plane_vertex, &quad);

            assert_eq!(transformed, case.expected);
        }
    }
}
