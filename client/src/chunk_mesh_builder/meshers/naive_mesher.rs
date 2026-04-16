use std::collections::HashMap;

use bevy::log::info_span;
use cgmath::Vector3;

use super::{ChunkMesher, MesherOutput, MesherWarning};
use crate::{
    chunk_mesh_builder::{
        meshers::MesherWarnings, ChunkMeshData, ChunkWithNeighbors, FaceData, MaterialId,
    }, voxel_render_core::RenderShape,
};
use shared::entities::*;

#[derive(Debug)]
pub struct NaiveMesher {
    render_shape_storage: Vec<RenderShape>,
}

impl ChunkMesher for NaiveMesher {
    fn create_mesh(&self, chunk: &ChunkWithNeighbors) -> MesherOutput {
        let _ = info_span!(
            "naive_mesher_create_mesh",
            name = "naive_mesher_create_mesh"
        )
        .entered();

        let mut out: HashMap<MaterialId, ChunkMeshData> = Default::default();
        let mut warnings: MesherWarnings = Default::default();
        let block_storage = chunk.get_origin_block_storage();

        for (index, block_id) in block_storage.iter().enumerate() {
            let pos = BlockInChunkPos::from_index(index);
            let result = self.render_shape_storage.get(*block_id as usize);

            match result {
                Some(render_shape) => {
                    let layer_mesh: &mut ChunkMeshData =
                        out.entry(render_shape.render_data().material).or_default();

                    self.create_block(
                        render_shape,
                        BlockInChunkPos::new(pos.x, pos.y, pos.z),
                        layer_mesh,
                        chunk,
                        &mut warnings,
                    );
                }
                None => {
                    warnings
                        .entry(MesherWarning::UnknownRenderShape(*block_id))
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
            }
        }

        MesherOutput {
            layers: out,
            warnings,
        }
    }
}

impl NaiveMesher {
    pub fn new(render_shape_storage: Vec<RenderShape>) -> Self {
        Self {
            render_shape_storage,
        }
    }

    fn create_block(
        &self,
        render_shape: &RenderShape,
        pos: BlockInChunkPos,
        out: &mut ChunkMeshData,
        chunk: &ChunkWithNeighbors,
        warnings: &mut MesherWarnings,
    ) {
        if !render_shape.render_data().visible {
            return;
        }

        for side in BlockSide::iterator().copied() {
            let result = self.has_translucent_neighbor(side, pos, chunk);

            let has_transparent_neighbor = match result {
                Err(err) => {
                    warnings.entry(err).and_modify(|e| *e += 1).or_insert(1);
                    true
                }
                Ok(has_transparent_neighbor) => has_transparent_neighbor,
            };

            if has_transparent_neighbor {
                out.faces.push(self.create_face(side, pos, render_shape));
            }
        }
    }

    fn has_translucent_neighbor(
        &self,
        side: BlockSide,
        pos: BlockInChunkPos,
        chunk: &ChunkWithNeighbors,
    ) -> Result<bool, MesherWarning> {
        let neighbor_pos = self.get_neighbor_pos(pos, side);
        let neighbor_id: BlockID = chunk.get(neighbor_pos);
        let neighbor_render_shape: &RenderShape = self
            .render_shape_storage
            .get(neighbor_id as usize)
            .ok_or(MesherWarning::UnknownRenderShape(neighbor_id))?;

        Ok(neighbor_render_shape.render_data().translucent)
    }

    fn get_neighbor_pos(&self, pos: BlockInChunkPos, side: BlockSide) -> Vector3<isize> {
        let dir = Direction::from(side);

        let x = pos.x as isize + dir.x;
        let y = pos.y as isize + dir.y;
        let z = pos.z as isize + dir.z;

        Vector3::<isize>::new(x, y, z)
    }

    fn create_face(
        &self,
        side: BlockSide,
        block_pos: BlockInChunkPos,
        render_shape: &RenderShape,
    ) -> FaceData {
        FaceData {
            facing_side: side,
            block_pos,
            uv_2: render_shape.get_storage_index(side).unwrap_or_default(),
        }
    }
}
