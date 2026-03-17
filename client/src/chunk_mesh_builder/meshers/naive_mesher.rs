use std::collections::HashMap;

use bevy::log::info_span;

use super::{ChunkMesher, MesherOutput, MesherWarning};
use crate::{
    bevy_resources::RenderShapeStorage,
    chunk_mesh_builder::{meshers::MesherWarnings, ChunkMeshData, MaterialId, Quad, RenderShape},
};
use shared::entities::*;

#[derive(Debug)]
pub struct NaiveMesher {
    render_shape_storage: RenderShapeStorage,
}

impl ChunkMesher for NaiveMesher {
    fn create_mesh(&self, chunk: &Chunk) -> MesherOutput {
        let _ = info_span!(
            "naive_mesher_create_mesh",
            name = "naive_mesher_create_mesh"
        )
        .entered();

        let mut out: HashMap::<MaterialId, ChunkMeshData> = Default::default();
        let mut warnings: MesherWarnings = Default::default();
        let block_storage = chunk.get_block_storage();

        for (index, block_id) in block_storage.iter().enumerate() {
            let pos = BlockInChunkPos::from_index(index);
            let result = self.render_shape_storage.get_by_id(*block_id as usize);

            match result {
                Some(render_shape) => {
                    let layer_mesh: &mut ChunkMeshData = out
                        .entry(render_shape.render_data().material)
                        .or_default();

                    self.create_block(
                        render_shape,
                        BlockInChunkPos::new(pos.x, pos.y, pos.z),
                        layer_mesh,
                        block_storage,
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
    pub fn new(render_shape_storage: RenderShapeStorage) -> Self {
        Self {
            render_shape_storage,
        }
    }

    fn create_block(
        &self,
        render_shape: &RenderShape,
        pos: BlockInChunkPos,
        out: &mut ChunkMeshData,
        block_storage: &BlockStorage,
        warnings: &mut MesherWarnings,
    ) {
        if !render_shape.render_data().visible {
            return;
        }

        for side in BlockSide::iterator().copied() {
            let result = self.has_translucent_neighbor(side, pos, block_storage);

            let has_transparent_neighbor = match result {
                Err(err) => {
                    warnings.entry(err).and_modify(|e| *e += 1).or_insert(1);
                    true
                }
                Ok(has_transparent_neighbor) => has_transparent_neighbor,
            };

            if has_transparent_neighbor {
                let quad = self.create_block_side(side, pos, render_shape);
                out.quads.push(quad);
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
            let neighbor_render_shape: &RenderShape = self
                .render_shape_storage
                .get_by_id(neighbor_id as usize)
                .ok_or(MesherWarning::UnknownRenderShape(neighbor_id))?;

            return Ok(neighbor_render_shape.render_data().translucent);
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
        render_shape: &RenderShape,
    ) -> Quad {
        
        Quad {
            facing_side: side,
            block_pos,
            uv_2: render_shape.get_storage_index(side).unwrap_or_default(),
        }
    }
}
