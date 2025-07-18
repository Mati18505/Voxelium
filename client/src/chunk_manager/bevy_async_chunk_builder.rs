use std::collections::HashMap;
use std::fmt;

use bevy::tasks::futures_lite::future;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::{prelude::*, tasks::Task};
use shared::chunk_io::pending_chunk_queue::PendingChunkQueue;
use shared::entities::{Chunk, ChunkPos};

use super::{chunk_state_manager, physical_world::Version};
use crate::chunk_mesh_builder::{meshers::naive_mesher::VoxelMesher, ChunkMesh};

#[derive(Resource)]
struct ChunkBuildTask(Task<ChunkMesh>);

pub struct AsyncChunkBuilder {
    voxel_mesher: VoxelMesher,
    pending_chunk_queue: PendingChunkQueue,
    chunks_to_build: HashMap<ChunkPos, (Chunk, Version)>,
    tasks: HashMap<(ChunkPos, Version), ChunkBuildTask>,
    completed: HashMap<(ChunkPos, Version), ChunkMesh>,
}

impl AsyncChunkBuilder {
    const MAX_BUILD_JOBS: usize = 16;

    pub fn new(voxel_mesher: VoxelMesher) -> Self {
        Self {
            voxel_mesher,
            pending_chunk_queue: PendingChunkQueue::new(),
            chunks_to_build: HashMap::default(),
            tasks: HashMap::default(),
            completed: HashMap::default(),
        }
    }

    fn create_build_task(&self, chunk: Chunk) -> Task<ChunkMesh> {
        let block_storage = chunk.get_block_storage().clone();
        let mut voxel_mesher = self.voxel_mesher.clone();

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move {
            let chunk_mesh = voxel_mesher.create_mesh(&block_storage).clone();

            if let Some(err) = voxel_mesher.get_last_err() {
                eprintln!("{err}");
            }

            chunk_mesh
        });

        task
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
}

impl chunk_state_manager::ChunkBuilder for AsyncChunkBuilder {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, version: Version) {
        self.pending_chunk_queue.add_chunk(chunk_pos);
        self.chunks_to_build
            .insert(chunk_pos, (chunk.clone(), version));
    }

    /// Should be called once per frame.
    fn update(&mut self, player_pos: ChunkPos) {
        // TODO
        // Chunk Grouping (for each thread job give multiple chunks).

        let nearest_chunks = self
            .pending_chunk_queue
            .take_nearest_chunks(Self::MAX_BUILD_JOBS, player_pos);

        for pos in nearest_chunks {
            let (chunk, version) = self
                .chunks_to_build
                .remove(&pos)
                .expect("Pending chunk not found in chunks_to_build");

            let task = self.create_build_task(chunk);
            self.tasks.insert((pos, version), ChunkBuildTask(task));
        }

        self.collect_finished_results();
    }

    fn take_built_chunk_mesh_by_version(
        &mut self,
        chunk_pos: ChunkPos,
        version: Version,
    ) -> Option<ChunkMesh> {
        self.completed.remove(&(chunk_pos, version))
    }

    fn is_chunk_mesh_built_with_version(&self, chunk_pos: ChunkPos, version: Version) -> bool {
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
