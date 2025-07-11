use std::collections::HashMap;
use std::fmt;

use bevy::tasks::futures_lite::future;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::{prelude::*, tasks::Task};
use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::{ChunkMesh, VoxelMesher};
use super::{physical_world::Version, chunk_state_manager};

#[derive(Resource)]
struct ChunkBuildTask(Task<ChunkMesh>);

pub struct AsyncChunkBuilder {
    voxel_mesher: VoxelMesher,
    tasks: HashMap<(ChunkPos, Version), ChunkBuildTask>,
    completed: HashMap<(ChunkPos, Version), ChunkMesh>,
}

impl AsyncChunkBuilder {
    pub fn new(voxel_mesher: VoxelMesher) -> Self {
        Self {
            voxel_mesher,
            tasks: HashMap::default(),
            completed: HashMap::default(),
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

            chunk_mesh
        });

        self.tasks.insert((chunk_pos, version), ChunkBuildTask(task));
    }
    
    fn collect_finished_results(&mut self) {
        let mut completed: HashMap<(ChunkPos, Version), ChunkMesh> = HashMap::default();

        for ((chunk_pos, version), build_task) in self.tasks.iter_mut() {
            if let Some(chunk_mesh) = future::block_on(future::poll_once(&mut build_task.0)) {
                completed.insert((*chunk_pos, *version), chunk_mesh);
            }
        }

        for (chunk_pos, version) in completed.keys() {
            self.tasks.remove(&(*chunk_pos, *version));
        }

        self.completed.extend(completed);
    }

    fn take_built_chunk_mesh_by_version(&mut self, chunk_pos: ChunkPos, version: Version) -> Option<ChunkMesh> {
        self.completed.remove(&(chunk_pos, version))
    }

    fn is_chunk_mesh_built_with_version(&mut self, chunk_pos: ChunkPos, version: Version) -> bool {
        self.completed.contains_key(&(chunk_pos, version))
    }
}

impl fmt::Debug for AsyncChunkBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncChunkBuilder")
            .field("tasks", &self.tasks.len())
            .field("completed", &self.completed.len())
            .finish()
    }
}