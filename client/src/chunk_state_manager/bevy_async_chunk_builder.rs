use std::collections::HashMap;

use bevy::tasks::futures_lite::future;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::{prelude::*, tasks::Task};
use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::{ChunkMesh, VoxelMesher};
use super::{physical_world::Version, chunk_state_manager};

#[derive(Resource)]
struct ChunkBuildTask(Task<(ChunkMesh, Version)>);

pub struct AsyncChunkBuilder {
    voxel_mesher: VoxelMesher,
    tasks: HashMap<ChunkPos, ChunkBuildTask>,
}

impl AsyncChunkBuilder {
    pub fn new(voxel_mesher: VoxelMesher) -> Self {
        Self {
            voxel_mesher,
            tasks: HashMap::default(),
        }
    }
}

impl chunk_state_manager::ChunkBuilder for AsyncChunkBuilder {
    fn build_chunk(
        &mut self,
        chunk_pos: ChunkPos,
        chunk: &Chunk,
        version: Version,
    ) {
        // TODO
        // Chunk Grouping (for each thread job give multiple chunks).

        let block_storage = chunk.get_block_storage().clone();
        let mut voxel_mesher = self.voxel_mesher.clone();

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move {
            let chunk_mesh = voxel_mesher.create_mesh(&block_storage).clone();

            if let Some(err) = voxel_mesher.get_last_err() {
                eprintln!("{}", err);
            }

            (chunk_mesh, version)
        });

        self.tasks.insert(chunk_pos, ChunkBuildTask(task));
    }

    fn take_builded_chunks(&mut self) -> HashMap<ChunkPos, (ChunkMesh, Version)> {
        let mut completed: HashMap<ChunkPos, (ChunkMesh, Version)> = HashMap::default();

        for (chunk_pos, build_task) in self.tasks.iter_mut() {
            if let Some((chunk_mesh, version)) = future::block_on(future::poll_once(&mut build_task.0)) {
                completed.insert(*chunk_pos, (chunk_mesh, version));
            }
        }

        for chunk_pos in completed.keys() {
            self.tasks.remove(chunk_pos);
        }

        completed
    }
}