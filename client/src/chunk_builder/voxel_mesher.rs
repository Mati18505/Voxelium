use std::{alloc::LayoutError, collections::HashMap, rc::Rc};

use bevy::render::view::Layer;
use cgmath::{ElementWise, Vector3};
use shared::entities::{
    BlockID, BlockInChunkPos, BlockSide, BlockStorage, Chunk, Direction, CHUNK_SIZE,
};

use super::{BlockTypeStorage, ChunkMesh, LayerMesh, MeshBlockType, TextureDictionary};

pub struct VoxelMesher {
    block_storage: BlockStorage,
    chunk_mesh: ChunkMesh,
    block_type_storage: Rc<BlockTypeStorage>,
    texture_dictionary: Rc<TextureDictionary>,
}

impl VoxelMesher {
    pub fn new(
        block_storage: BlockStorage,
        block_type_storage: Rc<BlockTypeStorage>,
        texture_dictionary: Rc<TextureDictionary>,
    ) -> Self {
        VoxelMesher {
            block_storage,
            chunk_mesh: ChunkMesh::default(),
            block_type_storage,
            texture_dictionary,
        }
    }

    pub fn create_mesh(&mut self) -> ChunkMesh {
        let mut chunk_mesh = ChunkMesh::default();

        for (index, block_id) in self.block_storage.iter().enumerate() {
            let pos = BlockInChunkPos::from_index(index);
            let result = self.block_type_storage.get_block_type_from_id(*block_id);

            match result {
                Some(block_type) => {
                    let mut layer_mesh: &mut LayerMesh = chunk_mesh
                        .layers
                        .entry(block_type.material_name.clone())
                        .or_insert(LayerMesh::default());

                    self.create_block(
                        block_type,
                        BlockInChunkPos::new(pos.x, pos.y, pos.z),
                        &mut layer_mesh,
                    );
                }
                None => {
                    eprintln!(
                        "Voxel mesher encountered unknown block type. BlockId = {}",
                        block_id
                    );
                    continue;
                }
            }
        }

        chunk_mesh
    }

    fn create_block(&self, block_type: &MeshBlockType, pos: BlockInChunkPos, mesh: &mut LayerMesh) {
        use BlockSide::*;
        if !block_type.is_visible {
            return;
        }

        for side in [Top, Bottom, Left, Right, Front, Back] {
            if self.has_transparent_neighbor(side, pos) {
                self.create_block_side(side, pos, &block_type, mesh);
            }
        }
    }

    fn has_transparent_neighbor(&self, side: BlockSide, pos: BlockInChunkPos) -> bool {
        if let Some(neighbor_pos) = self.get_neighbor_pos(pos, side) {
            let neighbor_id: BlockID = self.block_storage.get_block(neighbor_pos);
            let result = self.block_type_storage.get_block_type_from_id(neighbor_id);

            return match result {
                Some(neighbor_block_type) => neighbor_block_type.is_translucent,
                None => {
                    eprintln!(
                        "Voxel mesher encountered unknown block type. BlockId = {}",
                        neighbor_id
                    );
                    true
                }
            };
        }

        true
    }

    fn get_neighbor_pos(&self, pos: BlockInChunkPos, side: BlockSide) -> Option<BlockInChunkPos> {
        let neighbor_dir = Direction::from(side);

        pos.checked_add(neighbor_dir)
    }

    fn create_block_side(
        &self,
        side: BlockSide,
        pos: BlockInChunkPos,
        block_type: &MeshBlockType,
        mesh: &mut LayerMesh,
    ) {
        let pos_x = pos.x as f32;
        let pos_y = pos.y as f32;
        let pos_z = pos.z as f32;
        let pos = Vector3::new(pos_x, pos_y, pos_z);

        let front_vertices: [Vector3<f32>; 4] = [
            [-0.5, 0.5, -0.5].into(),
            [0.5, 0.5, -0.5].into(),
            [0.5, 0.5, 0.5].into(),
            [-0.5, 0.5, 0.5].into(),
        ];
        let back_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, -0.5].into(),
            [0.5, -0.5, -0.5].into(),
            [0.5, -0.5, 0.5].into(),
            [-0.5, -0.5, 0.5].into(),
        ];
        let right_vertices: [Vector3<f32>; 4] = [
            [0.5, -0.5, -0.5].into(),
            [0.5, -0.5, 0.5].into(),
            [0.5, 0.5, 0.5].into(),
            [0.5, 0.5, -0.5].into(),
        ];
        let left_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, -0.5].into(),
            [-0.5, -0.5, 0.5].into(),
            [-0.5, 0.5, 0.5].into(),
            [-0.5, 0.5, -0.5].into(),
        ];
        let top_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, 0.5].into(),
            [-0.5, 0.5, 0.5].into(),
            [0.5, 0.5, 0.5].into(),
            [0.5, -0.5, 0.5].into(),
        ];
        let bottom_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, -0.5].into(),
            [-0.5, 0.5, -0.5].into(),
            [0.5, 0.5, -0.5].into(),
            [0.5, -0.5, -0.5].into(),
        ];

        let front_triangles: [usize; 6] = [0, 3, 1, 1, 3, 2];
        let back_triangles: [usize; 6] = [0, 1, 3, 1, 2, 3];
        let right_triangles: [usize; 6] = [0, 3, 1, 1, 3, 2];
        let left_triangles: [usize; 6] = [0, 1, 3, 1, 2, 3];
        let top_triangles: [usize; 6] = [0, 3, 1, 1, 3, 2];
        let bottom_triangles: [usize; 6] = [0, 1, 3, 1, 2, 3];

        let front_uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let back_uvs = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let right_uvs = [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]];
        let left_uvs = [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]];
        let top_uvs = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let bottom_uvs = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

        match side {
            BlockSide::Top => {
                mesh.vertices.push((pos + top_vertices[0]).into());
                mesh.vertices.push((pos + top_vertices[1]).into());
                mesh.vertices.push((pos + top_vertices[2]).into());
                mesh.vertices.push((pos + top_vertices[3]).into());
            }
            BlockSide::Bottom => {
                mesh.vertices.push((pos + bottom_vertices[0]).into());
                mesh.vertices.push((pos + bottom_vertices[1]).into());
                mesh.vertices.push((pos + bottom_vertices[2]).into());
                mesh.vertices.push((pos + bottom_vertices[3]).into());
            }
            BlockSide::Left => {
                mesh.vertices.push((pos + left_vertices[0]).into());
                mesh.vertices.push((pos + left_vertices[1]).into());
                mesh.vertices.push((pos + left_vertices[2]).into());
                mesh.vertices.push((pos + left_vertices[3]).into());
            }
            BlockSide::Right => {
                mesh.vertices.push((pos + right_vertices[0]).into());
                mesh.vertices.push((pos + right_vertices[1]).into());
                mesh.vertices.push((pos + right_vertices[2]).into());
                mesh.vertices.push((pos + right_vertices[3]).into());
            }
            BlockSide::Front => {
                mesh.vertices.push((pos + front_vertices[0]).into());
                mesh.vertices.push((pos + front_vertices[1]).into());
                mesh.vertices.push((pos + front_vertices[2]).into());
                mesh.vertices.push((pos + front_vertices[3]).into());
            }
            BlockSide::Back => {
                mesh.vertices.push((pos + back_vertices[0]).into());
                mesh.vertices.push((pos + back_vertices[1]).into());
                mesh.vertices.push((pos + back_vertices[2]).into());
                mesh.vertices.push((pos + back_vertices[3]).into());
            }
        }

        let side_dir = Direction::from(side);

        for _ in 0..4 {
            mesh.normals.push([side_dir.x, side_dir.z, side_dir.y]);
        }

        let uvs = match side {
            BlockSide::Front => front_uvs,
            BlockSide::Back => back_uvs,
            BlockSide::Left => left_uvs,
            BlockSide::Right => right_uvs,
            BlockSide::Top => top_uvs,
            BlockSide::Bottom => bottom_uvs,
        };

        for uv in uvs {
            mesh.uvs.push(uv);
        }

        for _ in 0..4 {
            let texture_name = block_type.get_block_side_texture(side);
            let texture_index = self
                .texture_dictionary
                .get_texture_index_from_name(texture_name);

            mesh.texture_indexes.push(texture_index);
        }

        for i in 0..6 {
            match side {
                BlockSide::Front => mesh.triangles.push(mesh.vertex_index + front_triangles[i]),
                BlockSide::Back => mesh.triangles.push(mesh.vertex_index + back_triangles[i]),
                BlockSide::Left => mesh.triangles.push(mesh.vertex_index + left_triangles[i]),
                BlockSide::Right => mesh.triangles.push(mesh.vertex_index + right_triangles[i]),
                BlockSide::Top => mesh.triangles.push(mesh.vertex_index + top_triangles[i]),
                BlockSide::Bottom => mesh.triangles.push(mesh.vertex_index + bottom_triangles[i]),
            }
        }

        mesh.vertex_index += 4;
    }
}
