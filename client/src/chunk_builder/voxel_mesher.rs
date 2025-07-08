use std::{fmt, sync::Arc};

use cgmath::Vector3;
use shared::entities::{BlockID, BlockInChunkPos, BlockSide, BlockStorage, Direction};

use super::{
    BlockTypeStorage, ChunkMesh, LayerMesh, MeshBlockType, TextureDictionary, TextureName,
};

#[derive(Debug, Clone)]
pub struct VoxelMesher {
    block_type_storage: Arc<BlockTypeStorage>,
    texture_dictionary: Arc<TextureDictionary>,
    last_error: Option<MesherError>,
}

#[derive(Debug, Clone)]
pub enum MesherError {
    UnknownBlockType(BlockID),
    UnknownTextureName(TextureName),
}

impl fmt::Display for MesherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::UnknownBlockType(block_id) => {
                format!("Voxel mesher encountered unknown block type. BlockId = {block_id}")
            }
            Self::UnknownTextureName(texture_name) => {
                format!(
                    "Voxel mesher encountered unknown texture name. texture_name = {texture_name}"
                )
            }
        };
        write!(f, "{}", s)
    }
}

impl VoxelMesher {
    pub fn new(
        block_type_storage: Arc<BlockTypeStorage>,
        texture_dictionary: Arc<TextureDictionary>,
    ) -> Self {
        VoxelMesher {
            block_type_storage,
            texture_dictionary,
            last_error: None,
        }
    }

    pub fn create_mesh(
        &mut self,
        block_storage: &BlockStorage,
    ) -> ChunkMesh {
        let mut chunk_mesh = ChunkMesh::default();
        let mut last_err = None;

        for (index, block_id) in block_storage.iter().enumerate() {
            let pos = BlockInChunkPos::from_index(index);
            let result = self.block_type_storage.get_block_type_from_id(*block_id);

            match result {
                Some(block_type) => {
                    let mut layer_mesh: &mut LayerMesh = chunk_mesh
                        .layers
                        .entry(block_type.material_name.clone())
                        .or_insert(LayerMesh::default());

                    let result = self.create_block(
                        block_type,
                        BlockInChunkPos::new(pos.x, pos.y, pos.z),
                        &mut layer_mesh,
                        block_storage,
                    );

                    if let Err(err) = result {
                        last_err = Some(err);
                    }
                }
                None => {
                    last_err = Some(MesherError::UnknownBlockType(*block_id));
                    continue;
                }
            }
        }

        if let Some(err) = last_err {
            self.set_last_err(err);
        }

        chunk_mesh
    }

    pub fn get_last_err(&self) -> &Option<MesherError> {
        &self.last_error
    }

    fn set_last_err(&mut self, err: MesherError) {
        self.last_error = Some(err);
    }

    fn create_block(
        &self,
        block_type: &MeshBlockType,
        pos: BlockInChunkPos,
        mesh: &mut LayerMesh,
        block_storage: &BlockStorage,
    ) -> Result<(), MesherError> {
        use BlockSide::*;
        if !block_type.is_visible {
            return Ok(());
        }

        let mut last_err = None;

        for side in [Top, Bottom, Left, Right, Front, Back] {
            let result = self.has_translucent_neighbor(side, pos, block_storage);

            let has_transparent_neighbor = match result {
                Err(err) => {
                    last_err = Some(err);
                    true
                }
                Ok(has_transparent_neighbor) => has_transparent_neighbor,
            };

            if has_transparent_neighbor {
                let result = self.create_block_side(side, pos, &block_type, mesh);

                if let Err(err) = result {
                    last_err = Some(err);
                }
            }
        }

        if let Some(err) = last_err {
            Err(err)
        } else {
            Ok(())
        }
    }

    fn has_translucent_neighbor(
        &self,
        side: BlockSide,
        pos: BlockInChunkPos,
        block_storage: &BlockStorage,
    ) -> Result<bool, MesherError> {
        if let Some(neighbor_pos) = self.get_neighbor_pos(pos, side) {
            let neighbor_id: BlockID = block_storage.get_block(neighbor_pos);
            let neighbor_block_type: &MeshBlockType = self
                .block_type_storage
                .get_block_type_from_id(neighbor_id)
                .ok_or(MesherError::UnknownBlockType(neighbor_id))?;

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
        pos: BlockInChunkPos,
        block_type: &MeshBlockType,
        mesh: &mut LayerMesh,
    ) -> Result<(), MesherError> {
        let mut last_err = None;

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

        let front_uvs = [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]];
        let back_uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        let right_uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let left_uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
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
            let result = self
                .texture_dictionary
                .get_texture_index_from_name(texture_name);

            let texture_index: u32 = match result {
                Some(index) => index,
                None => {
                    last_err = Some(MesherError::UnknownTextureName(texture_name.to_owned()));
                    0
                }
            };

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

        if let Some(err) = last_err {
            Err(err)
        } else {
            Ok(())
        }
    }
}
