use std::collections::HashMap;

use bevy::tasks::futures_lite::future;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::{prelude::*, tasks::Task};
use shared::entities::{Chunk, ChunkPos};

use crate::chunk_builder::{ChunkMesh, VoxelMesher};
use crate::chunk_manager::chunk_manager;

#[derive(Resource)]
struct ChunkBuildTask(Task<ChunkMesh>);

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

impl chunk_manager::ChunkBuilder for AsyncChunkBuilder {
    fn build_chunk(
        &mut self,
        chunk_pos: ChunkPos,
        chunk: &Chunk,
    ) {
        assert!(!self.tasks.contains_key(&chunk_pos), "Can't build a fragment at the same position a second time.");

        // TODO
        // Chunk Grouping (for each thread job give multiple chunks).
        // What if chunk needs to be redrawn before being build? Assert will crash the application.

        let block_storage = chunk.get_block_storage().clone();
        let mut voxel_mesher = self.voxel_mesher.clone();

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move {
            let chunk_mesh = voxel_mesher.create_mesh(&block_storage).clone();

            if let Some(err) = voxel_mesher.get_last_err() {
                eprintln!("{}", err);
            }

            chunk_mesh
        });

        self.tasks.insert(chunk_pos, ChunkBuildTask(task));
    }

    fn get_builded_chunks(&mut self) -> HashMap<ChunkPos, ChunkMesh> {
        let mut completed: HashMap<ChunkPos, ChunkMesh> = HashMap::default();

        for (chunk_pos, build_task) in self.tasks.iter_mut() {
            if let Some(chunk_mesh) = future::block_on(future::poll_once(&mut build_task.0)) {
                completed.insert(*chunk_pos, chunk_mesh);
            }
        }

        for chunk_pos in completed.keys() {
            self.tasks.remove(chunk_pos);
        }

        completed
    }
}