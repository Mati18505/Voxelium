use bevy::{
    prelude::*,
    tasks::{futures_lite::future, AsyncComputeTaskPool, Task},
};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    sync::Arc,
};

use crate::chunk_mesh_builder::{builders::ChunkBuilder, meshers::ChunkMesher, ChunkMesh};
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{Chunk, ChunkPos},
};

// struct ChunkBuildTask<T: Send + Sync + Default>(Task<ChunkMesh>);
#[derive(Resource)]
struct ChunkBuildTask(Task<ChunkMesh>);

pub struct AsyncChunkBuilder<T>
where
    T: Send + Sync + Default,
{
    mesher: Arc<dyn ChunkMesher>,
    pending_chunk_queue: PendingChunkQueue,
    chunks_to_build: Vec<(ChunkPos, (Chunk, T))>,
    tasks: Vec<(ChunkPos, (ChunkBuildTask, T))>,
    completed: Vec<(ChunkPos, (ChunkMesh, T))>,
}

impl<T: Send + Sync + Default + Debug> AsyncChunkBuilder<T> {
    const MAX_BUILD_JOBS: usize = 16;

    pub fn new(mesher: Arc<dyn ChunkMesher>) -> Self {
        Self {
            mesher,
            pending_chunk_queue: PendingChunkQueue::new(),
            chunks_to_build: Vec::default(),
            tasks: Vec::default(),
            completed: Vec::default(),
        }
    }

    fn create_build_task(&self, chunk: Chunk) -> Task<ChunkMesh> {
        let mesher = self.mesher.clone();

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move {
            let mesher_result = mesher.create_mesh(&chunk).clone();

            for warning in mesher_result.warnings {
                warn!("{warning}");
            }

            mesher_result.mesh
        });

        task
    }

    fn collect_finished_results(&mut self) {
        let mut completed: HashMap<ChunkPos, (ChunkMesh, T)> = HashMap::default();

        for (chunk_pos, (build_task, additional_data)) in self.tasks.iter_mut() {
            if let Some(chunk_mesh) = future::block_on(future::poll_once(&mut build_task.0)) {
                completed.insert(*chunk_pos, (chunk_mesh, std::mem::take(additional_data)));
            }
        }

        for pos in completed.keys() {
            self.tasks.retain(|(chunk_pos, _)| chunk_pos != pos);
        }

        self.completed.extend(completed);
    }

    fn take_chunks_to_build(&mut self, pos: ChunkPos) -> Vec<(ChunkPos, (Chunk, T))> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < self.chunks_to_build.len() {
            if self.chunks_to_build[i].0 == pos {
                let e = self.chunks_to_build.swap_remove(i);
                result.push(e);
            } else {
                i += 1;
            }
        }

        result
    }
}

impl<T: Send + Sync + Default + Debug> ChunkBuilder<T> for AsyncChunkBuilder<T> {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: T) {
        self.pending_chunk_queue.add_chunk(chunk_pos);
        self.chunks_to_build
            .push((chunk_pos, (chunk.clone(), additional_data)));
    }

    fn update(&mut self, player_pos: ChunkPos) {
        // TODO
        // Chunk Grouping (for each thread job give multiple chunks).

        let nearest_chunks = self
            .pending_chunk_queue
            .take_nearest_chunks(Self::MAX_BUILD_JOBS, player_pos);

        for pos in nearest_chunks {
            let chunks = self.take_chunks_to_build(pos);

            for (pos, (chunk, additional_data)) in chunks {
                let task = self.create_build_task(chunk);
                self.tasks
                    .push((pos, (ChunkBuildTask(task), additional_data)));
            }
        }

        self.collect_finished_results();
    }

    fn remove_chunk(&mut self, pos: ChunkPos) {
        self.pending_chunk_queue.remove_chunk(pos);
        self.chunks_to_build
            .retain(|(chunk_pos, _)| *chunk_pos != pos);
        self.tasks.retain(|(chunk_pos, _)| *chunk_pos != pos);
        self.completed.retain(|(chunk_pos, _)| *chunk_pos != pos);
    }

    fn clear_all(&mut self) {
        self.pending_chunk_queue = PendingChunkQueue::default();
        self.chunks_to_build.clear();
        self.tasks.clear();
        self.completed.clear();
    }

    fn poll_completed(&mut self) -> Vec<(ChunkPos, (ChunkMesh, T))> {
        std::mem::take(&mut self.completed)
    }
}

impl<T: Send + Sync + Default> fmt::Debug for AsyncChunkBuilder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncChunkBuilder")
            .field("pending_chunk_queue", &self.pending_chunk_queue)
            .field("to_build", &self.chunks_to_build.len())
            .field("tasks", &self.tasks.len())
            .field("completed", &self.completed.len())
            .finish()
    }
}
