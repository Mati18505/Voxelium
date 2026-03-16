use bevy::{
    prelude::*,
    tasks::{futures_lite::future, AsyncComputeTaskPool, Task},
};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    sync::Arc,
};

use crate::{
    bevy_resources::BlockTypeName,
    chunk_mesh_builder::{
        builders::ChunkBuilder,
        meshers::{ChunkMesher, MesherOutput, MesherWarning, MesherWarnings},
        ChunkMesh,
    },
};
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{iterate_over_block_registry, BlockID, Chunk, ChunkPos},
};

use super::BuilderError;

// struct ChunkBuildTask<T: Send + Sync + Default>(Task<ChunkMesh>);
#[derive(Resource)]
struct ChunkBuildTask(Task<(ChunkMesh, MesherWarnings)>);

pub struct AsyncChunkBuilder<T>
where
    T: Send + Sync + Default,
{
    mesher: Arc<dyn ChunkMesher>,
    pending_chunk_queue: PendingChunkQueue,
    chunks_to_build: HashMap<ChunkPos, (Chunk, T)>,
    tasks: HashMap<ChunkPos, (ChunkBuildTask, T)>,
    completed: HashMap<ChunkPos, (ChunkMesh, T)>,
    block_id_to_name: HashMap<BlockID, BlockTypeName>,
}

impl<T: Send + Sync + Default + Debug> AsyncChunkBuilder<T> {
    const MAX_BUILD_JOBS: usize = 16;

    pub fn new(mesher: Arc<dyn ChunkMesher>) -> Self {
        let block_id_to_name = iterate_over_block_registry()
            .map(|(name, block_id)| (*block_id, name.to_string()))
            .collect();

        Self {
            mesher,
            pending_chunk_queue: PendingChunkQueue::new(),
            chunks_to_build: HashMap::default(),
            tasks: HashMap::default(),
            completed: HashMap::default(),
            block_id_to_name,
        }
    }

    fn create_build_task(&self, chunk: Chunk) -> ChunkBuildTask {
        let mesher = self.mesher.clone();
        let pool = AsyncComputeTaskPool::get();

        let future = async move { 
            let mesh_data = mesher.create_mesh(&chunk).clone();
            let mesh = mesh_data.mesh.mesh().build();

            (mesh, mesh_data.warnings)
        };

        ChunkBuildTask(pool.spawn(future))
    }

    fn collect_finished_results(&mut self) {
        let mut completed: HashMap<ChunkPos, (ChunkMesh, T)> = HashMap::default();
        let mut warnings: HashMap<MesherWarning, u32> = HashMap::default();

        for (chunk_pos, (build_task, additional_data)) in self.tasks.iter_mut() {
            if let Some(out) = future::block_on(future::poll_once(&mut build_task.0)) {
                completed.insert(
                    *chunk_pos,
                    (out.0, std::mem::take(additional_data)),
                );

                for (warning, count) in out.1 {
                    warnings
                        .entry(warning)
                        .and_modify(|e| *e += count)
                        .or_insert(count);
                }
            }
        }

        for pos in completed.keys() {
            self.tasks.remove(pos);
        }

        self.completed.extend(completed);

        self.log_warnings(warnings);
    }

    fn is_chunk_in_builder(&self, chunk_pos: ChunkPos) -> bool {
        self.chunks_to_build.contains_key(&chunk_pos)
            || self.tasks.contains_key(&chunk_pos)
            || self.completed.contains_key(&chunk_pos)
    }

    fn log_warnings(&self, warnings: HashMap<MesherWarning, u32>) {
        for (warning, count) in warnings {
            let warn = match warning {
                MesherWarning::UnknownRenderShape(block_id) => {
                    let block_type_name = self
                        .block_id_to_name
                        .get(&block_id)
                        .cloned()
                        .unwrap_or("<unknown>".to_string());

                    format!("{warning} ({block_type_name}) - occured {count} times")
                }
            };
            warn!(warn);
        }
    }
}

impl<T: Send + Sync + Default + Debug> ChunkBuilder<T> for AsyncChunkBuilder<T> {
    fn force_build(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: T) {
        self.remove_chunk(chunk_pos);

        self.pending_chunk_queue.add_chunk(chunk_pos);
        self.chunks_to_build
            .insert(chunk_pos, (chunk.clone(), additional_data));
    }

    fn try_build(
        &mut self,
        chunk_pos: ChunkPos,
        chunk: &Chunk,
        additional_data: T,
    ) -> std::result::Result<(), BuilderError> {
        if self.is_chunk_in_builder(chunk_pos) {
            return Err(BuilderError::ChunkAlreadyExists(chunk_pos));
        }

        self.force_build(chunk_pos, chunk, additional_data);
        Ok(())
    }

    fn update(&mut self, player_pos: ChunkPos) {
        // TODO
        // Chunk Grouping (for each thread job give multiple chunks).

        let nearest_chunks = self
            .pending_chunk_queue
            .take_nearest_chunks(Self::MAX_BUILD_JOBS, player_pos);

        for pos in nearest_chunks {
            if let Some((chunk, additional_data)) = self.chunks_to_build.remove(&pos) {
                let task = self.create_build_task(chunk);
                self.tasks
                    .insert(pos, (task, additional_data));
            }
        }

        self.collect_finished_results();
    }

    fn remove_chunk(&mut self, pos: ChunkPos) {
        self.pending_chunk_queue.remove_chunk(pos);
        self.chunks_to_build.remove(&pos);
        self.tasks.remove(&pos);
        self.completed.remove(&pos);
    }

    fn clear_all(&mut self) {
        self.pending_chunk_queue = PendingChunkQueue::default();
        self.chunks_to_build.clear();
        self.tasks.clear();
        self.completed.clear();
    }

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)> {
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
