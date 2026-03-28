use cgmath::num_traits::pow;
use lazy_static::lazy_static;

use bevy::{
    asset::RenderAssetUsages,
    log::info_span,
    math::{UVec3, Vec3},
    mesh::{Indices, Mesh, MeshBuilder, MeshVertexAttribute, PrimitiveTopology, VertexFormat},
};
use shared::entities::{BlockSide, Direction, CHUNK_SIZE};

use crate::chunk_mesh_builder::{ChunkMeshData, FaceData};

pub const ATTRIBUTE_PACKED_DATA: MeshVertexAttribute = MeshVertexAttribute::new(
    "packed_data",
    Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE,
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

struct VertexData {
    pos_index: u32,
    normal: Normal,
    uv: UV,
    storage_index: u32,
}

impl VertexData {
    const POS_BITS: u32 = 13;
    const NORMAL_BITS: u32 = 3;
    const UV_BITS: u32 = 2;
    const SI_BITS: u32 = 8;

    const POS_MASK: u32 = (1 << Self::POS_BITS) - 1;
    const NORMAL_MASK: u32 = (1 << Self::NORMAL_BITS) - 1;
    const UV_MASK: u32 = (1 << Self::UV_BITS) - 1;
    const SI_MASK: u32 = (1 << Self::SI_BITS) - 1;

    fn new(pos_index: u32, normal: Normal, uv: UV, storage_index: u32) -> Self {
        let max_pos_index = pow(CHUNK_SIZE as u32 + 1, 3);

        debug_assert!(pos_index < max_pos_index);
        debug_assert!((normal as u32) <= Self::NORMAL_MASK);
        debug_assert!((uv as u32) <= Self::UV_MASK);
        debug_assert!(storage_index <= Self::SI_MASK);

        Self {
            pos_index,
            normal,
            uv,
            storage_index,
        }
    }

    fn pack(&self) -> u32 {
        let normal = self.normal as u32;
        let uv = self.uv as u32;

        let offsets = [
            Self::POS_BITS,
            (Self::POS_BITS + Self::NORMAL_BITS),
            (Self::POS_BITS + Self::NORMAL_BITS + Self::UV_BITS),
        ];

        (self.pos_index & Self::POS_MASK)
            | ((normal & Self::NORMAL_MASK) << offsets[0])
            | ((uv & Self::UV_MASK) << offsets[1])
            | ((self.storage_index & Self::SI_MASK) << offsets[2])
    }

    #[cfg(test)]
    fn unpack(packed: u32) -> Result<Self, ()> {
        let offsets = [
            Self::POS_BITS,
            (Self::POS_BITS + Self::NORMAL_BITS),
            (Self::POS_BITS + Self::NORMAL_BITS + Self::UV_BITS),
        ];

        let pos_index = packed & Self::POS_MASK;
        let normal = (packed >> offsets[0]) & Self::NORMAL_MASK;
        let uv = (packed >> offsets[1]) & Self::UV_MASK;
        let storage_index = (packed >> offsets[2]) & Self::SI_MASK;

        let normal = Normal::try_from(normal)?;
        let uv = UV::try_from(uv)?;

        Ok(Self {
            pos_index,
            normal,
            uv,
            storage_index,
        })
    }
}

impl ChunkMeshBuilder {
    /// Plane mesh builder without rotation, scale and subdivision.
    fn build_face(facing_side: BlockSide) -> MeshData {
        let z_vertex_count = 2;
        let x_vertex_count = 2;
        let num_vertices = z_vertex_count * x_vertex_count;
        let num_indices = (z_vertex_count - 1) * (x_vertex_count - 1) * 6;

        let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<Normal> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<UV> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        let uv_lookup = [UV::BottomRight, UV::BottomLeft, UV::TopRight, UV::TopLeft];

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
                uvs.push(uv_lookup[uv_index]);
            }
        }

        let z_vertex_count = z_vertex_count as u32;
        let x_vertex_count = x_vertex_count as u32;

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

        let mut packed_data: Vec<u32> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        for (i, quad) in self.chunk_mesh_data.faces.iter().enumerate() {
            let face = &FACES[quad.facing_side as usize];

            for (i, plane_pos) in face.positions.iter().enumerate() {
                let vertex_pos = Self::plane_pos_to_vertex_pos(plane_pos, quad);
                let pos_index = Self::vertex_pos_to_index(vertex_pos);

                let normal = face.normals.get(i).unwrap();
                let uv = face.uvs.get(i).unwrap();
                let storage_index = quad.uv_2;

                let vertex = VertexData::new(pos_index, *normal, *uv, storage_index);
                let packed = vertex.pack();

                packed_data.push(packed);
            }

            let base_index = 4 * i as u32;
            indices.extend(face.indices.iter().map(|e| *e + base_index));
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(ATTRIBUTE_PACKED_DATA, packed_data)
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

    #[test]
    fn test_pack_unpack_roundtrip() {
        let cases = [
            (0, 0, 0, 0),
            (1, 1, 1, 1),
            (3, 5, 2, 200),
            (pow(CHUNK_SIZE as u32 + 1, 3) - 1, 5, 3, VertexData::SI_MASK),
        ];

        for (pos_index, normal, uv, storage_index) in cases {
            let v = VertexData::new(
                pos_index,
                Normal::try_from(normal).unwrap(),
                UV::try_from(uv).unwrap(),
                storage_index,
            );

            let packed = v.pack();
            let unpacked = VertexData::unpack(packed).unwrap();

            assert_eq!(unpacked.pos_index, pos_index);
            assert_eq!(unpacked.normal as u32, normal);
            assert_eq!(unpacked.uv as u32, uv);
            assert_eq!(unpacked.storage_index, storage_index);
        }
    }

    #[test]
    fn test_vertex_data_out_of_range() {
        let cases = [
            (VertexData::POS_MASK + 1, 0, 0, 0),
            (0, 7, 0, 0),
            (0, 0, 4, 0),
            (0, 0, 0, VertexData::SI_MASK + 1),
        ];

        for (pos_index, normal, uv, storage_index) in cases {
            let result = std::panic::catch_unwind(|| {
                VertexData::new(
                    pos_index,
                    Normal::try_from(normal).unwrap(),
                    UV::try_from(uv).unwrap(),
                    storage_index,
                )
            });
            assert!(result.is_err());
        }
    }
}
