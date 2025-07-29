use std::sync::Arc;

use bevy::log::info_span;
use cgmath::Vector3;
use shared::entities::{BlockID, BlockInChunkPos, BlockSide, BlockStorage, Chunk, Direction};

use crate::chunk_mesh_builder::{
    ChunkMesh, LayerMesh, RenderBlockType, RenderBlockTypeStorage, TextureDictionary, TexturedBlockType
};

use super::{ChunkMesher, MesherOutput, MesherWarning};

#[derive(Debug, Clone)]
pub struct NaiveMesher {
    block_type_storage: Arc<RenderBlockTypeStorage>,
    texture_dictionary: Arc<TextureDictionary>,
}

impl ChunkMesher for NaiveMesher {
    fn create_mesh(&self, chunk: &Chunk) -> MesherOutput {
        let _ = info_span!(
            "naive_mesher_create_mesh",
            name = "naive_mesher_create_mesh"
        )
        .entered();

        let mut chunk_mesh = ChunkMesh::default();
        let mut warnings: Vec<MesherWarning> = Vec::default();
        let block_storage = chunk.get_block_storage();

        for (index, block_id) in block_storage.iter().enumerate() {
            let pos = BlockInChunkPos::from_index(index);
            let result = self.block_type_storage.get_block_type_from_id(*block_id);

            match result {
                Some(block_type) => {
                    let layer_mesh: &mut LayerMesh = chunk_mesh
                        .layers
                        .entry(block_type.material().clone())
                        .or_insert(LayerMesh::default());

                    self.create_block(
                        block_type,
                        BlockInChunkPos::new(pos.x, pos.y, pos.z),
                        layer_mesh,
                        block_storage,
                        &mut warnings,
                    );
                }
                None => {
                    warnings.push(MesherWarning::UnknownBlockType(*block_id, pos));
                }
            }
        }

        MesherOutput {
            mesh: chunk_mesh,
            warnings,
        }
    }
}

impl NaiveMesher {
    pub fn new(
        block_type_storage: Arc<RenderBlockTypeStorage>,
        texture_dictionary: Arc<TextureDictionary>,
    ) -> Self {
        NaiveMesher {
            block_type_storage,
            texture_dictionary,
        }
    }

    fn create_block(
        &self,
        block_type: &dyn RenderBlockType,
        pos: BlockInChunkPos,
        mesh: &mut LayerMesh,
        block_storage: &BlockStorage,
        warnings: &mut Vec<MesherWarning>,
    ) {
        use BlockSide::*;

        if !block_type.is_visible {
            return;
        }

        for side in [Top, Bottom, Left, Right, Front, Back] {
            let result = self.has_translucent_neighbor(side, pos, block_storage);

            let has_transparent_neighbor = match result {
                Err(err) => {
                    warnings.push(err);
                    true
                }
                Ok(has_transparent_neighbor) => has_transparent_neighbor,
            };

            if has_transparent_neighbor {
                self.create_block_side(side, pos, block_type, mesh, warnings);
            }
        }
    }

    fn has_translucent_neighbor(
        &self,
        side: BlockSide,
        pos: BlockInChunkPos,
        block_storage: &BlockStorage,
    ) -> Result<bool, MesherWarning> {
        if let Some(neighbor_pos) = self.get_neighbor_pos(pos, side) {
            let neighbor_id: BlockID = block_storage.get_block(neighbor_pos);
            let neighbor_block_type: &TexturedBlockType = self
                .block_type_storage
                .get_block_type_from_id(neighbor_id)
                .ok_or(MesherWarning::UnknownBlockType(neighbor_id, pos))?;

            return Ok(neighbor_block_type.is_translucent);
        }

        Ok(true)
    }

    fn get_neighbor_pos(&self, pos: BlockInChunkPos, side: BlockSide) -> Option<BlockInChunkPos> {
        let neighbor_dir = Direction::from(side);

        pos.checked_add(neighbor_dir)
    }

    fn create_block_side(
        &self,
        side: BlockSide,
        block_pos: BlockInChunkPos,
        block_type: &dyn RenderBlockType,
        mesh: &mut LayerMesh,
        warnings: &mut Vec<MesherWarning>,
    ) {
        let pos_x = block_pos.x as f32;
        let pos_y = block_pos.y as f32;
        let pos_z = block_pos.z as f32;
        let pos = Vector3::new(pos_x, pos_y, pos_z);

        let front_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, 0.5].into(), // 0: bottom-left
            [0.5, -0.5, 0.5].into(),  // 1: bottom-right
            [0.5, 0.5, 0.5].into(),   // 2: top-right
            [-0.5, 0.5, 0.5].into(),  // 3: top-left
        ];

        let back_vertices: [Vector3<f32>; 4] = [
            [0.5, -0.5, -0.5].into(),  // 0: bottom-left (mirror)
            [-0.5, -0.5, -0.5].into(), // 1: bottom-right
            [-0.5, 0.5, -0.5].into(),  // 2: top-right
            [0.5, 0.5, -0.5].into(),   // 3: top-left
        ];

        let right_vertices: [Vector3<f32>; 4] = [
            [0.5, -0.5, 0.5].into(),  // 0: bottom-left
            [0.5, -0.5, -0.5].into(), // 1: bottom-right
            [0.5, 0.5, -0.5].into(),  // 2: top-right
            [0.5, 0.5, 0.5].into(),   // 3: top-left
        ];

        let left_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, -0.5].into(), // 0: bottom-left
            [-0.5, -0.5, 0.5].into(),  // 1: bottom-right
            [-0.5, 0.5, 0.5].into(),   // 2: top-right
            [-0.5, 0.5, -0.5].into(),  // 3: top-left
        ];

        let top_vertices: [Vector3<f32>; 4] = [
            [-0.5, 0.5, 0.5].into(),  // 0: bottom-left
            [0.5, 0.5, 0.5].into(),   // 1: bottom-right
            [0.5, 0.5, -0.5].into(),  // 2: top-right
            [-0.5, 0.5, -0.5].into(), // 3: top-left
        ];

        let bottom_vertices: [Vector3<f32>; 4] = [
            [-0.5, -0.5, -0.5].into(), // 0: bottom-left
            [0.5, -0.5, -0.5].into(),  // 1: bottom-right
            [0.5, -0.5, 0.5].into(),   // 2: top-right
            [-0.5, -0.5, 0.5].into(),  // 3: top-left
        ];

        let triangles: [usize; 6] = [0, 1, 3, 1, 2, 3];

        let front_uvs = [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]];
        let back_uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
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
            mesh.normals.push([side_dir.x, side_dir.y, side_dir.z]);
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
            let result = self
                .texture_dictionary
                .get_texture_index_from_name(texture_name);

            let texture_index: u32 = match result {
                Some(index) => index,
                None => {
                    warnings.push(MesherWarning::UnknownTextureName(
                        texture_name.to_owned(),
                        block_pos,
                    ));
                    0
                }
            };

            mesh.texture_indexes.push(texture_index);
        }

        for t in triangles {
            mesh.triangles.push(mesh.vertex_index + t);
        }

        mesh.vertex_index += 4;
    }
}
