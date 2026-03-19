use std::{collections::HashMap, fmt, time::Duration};

use bevy::{
    prelude::*,
    tasks::{futures_lite::future, AsyncComputeTaskPool, Task},
};
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{iterate_over_block_registry, BlockID, Chunk, ChunkPos, ChunkRepository},
};

use crate::{
    bevy_resources::BlockTypeName,
    bevy_types::AppStates,
    chunk_manager::{ChunkMesherResource, ChunkStorage, ControllerPos},
    chunk_mesh_builder::{
        meshers::{MesherWarning, MesherWarnings},
        ChunkMesh,
    },
};

#[derive(Message, Debug, Clone, PartialEq)]
pub struct BuildChunk(pub ChunkPos);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct RemoveChunk(pub ChunkPos);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct ChunkBuilt(pub ChunkPos);

#[derive(Resource, Debug, Clone)]
pub struct ChunkBuilderConfig {
    pub max_build_tasks: usize,
}

pub struct ChunkBuilderPlugin(ChunkBuilderConfig);
impl ChunkBuilderPlugin {
    pub fn new(config: ChunkBuilderConfig) -> Self {
        Self(config)
    }
}
impl Plugin for ChunkBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .init_resource::<BuilderResources>()
            .insert_resource(DebugTimer(Timer::new(
                Duration::from_secs(2),
                TimerMode::Repeating,
            )))
            .add_message::<BuildChunk>()
            .add_message::<RemoveChunk>()
            .add_message::<ChunkBuilt>()
            .add_systems(
                Update,
                (
                    process_chunks_to_build,
                    add_tasks,
                    collect_finished,
                    debug_state,
                )
                    .run_if(in_state(AppStates::InGame)),
            );
    }
}

#[derive(Resource)]
struct ChunkBuildTask(Task<(ChunkMesh, MesherWarnings)>);

#[derive(Resource, Default)]
pub struct BuilderResources {
    chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
    tasks: HashMap<ChunkPos, ChunkBuildTask>,
    pending_chunk_queue: PendingChunkQueue,
}

impl fmt::Debug for BuilderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncChunkBuilder")
            .field("pending_chunk_queue", &self.pending_chunk_queue)
            .field("tasks", &self.tasks.len())
            .field("meshes", &self.chunk_meshes.len())
            .finish()
    }
}

#[derive(Resource)]
pub struct DebugTimer(Timer);

pub fn process_chunks_to_build(
    mut reader: MessageReader<BuildChunk>,
    mut data: ResMut<BuilderResources>,
) {
    for message in reader.read() {
        let chunk_pos = message.0;

        data.tasks.remove(&chunk_pos);
        data.chunk_meshes.remove(&chunk_pos);
        data.pending_chunk_queue.add_chunk(chunk_pos);
    }
}

pub fn process_chunks_to_remove(mut reader: MessageReader<RemoveChunk>) {
    for message in reader.read() {
        let chunk_pos = message.0;

        info!("Removing chunk {:?}", &chunk_pos);
    }
}

pub fn add_tasks(
    mut data: ResMut<BuilderResources>,
    controller_pos: Res<ControllerPos>,
    config: Res<ChunkBuilderConfig>,
    chunks: Res<ChunkStorage>,
    mesher: Res<ChunkMesherResource>,
) {
    // TODO
    // Chunk Grouping (for each thread task give multiple chunks).

    let nearest_chunks = data
        .pending_chunk_queue
        .take_nearest_chunks(config.max_build_tasks, controller_pos.0);

    for chunk_pos in nearest_chunks {
        if let Some(chunk) = chunks.get_chunk(chunk_pos) {
            let task = create_build_task(chunk, &mesher);

            data.tasks.insert(chunk_pos, task);
        } else {
            warn!("Chunk is passed to builder, but it doesn't exist in ChunkStorage.");
        }
    }
}

fn create_build_task(chunk: &Chunk, mesher: &ChunkMesherResource) -> ChunkBuildTask {
    let chunk = chunk.clone();
    let mesher = mesher.0.clone();
    let pool = AsyncComputeTaskPool::get();

    let future = async move {
        let mesh_data = mesher.create_mesh(&chunk);
        let mut chunk_mesh: ChunkMesh = Default::default();

        for (material_id, mesh) in mesh_data.layers.iter() {
            chunk_mesh.layers.insert(*material_id, mesh.mesh().build());
        }

        (chunk_mesh, mesh_data.warnings)
    };

    ChunkBuildTask(pool.spawn(future))
}

fn collect_finished(
    mut data: ResMut<BuilderResources>,
    mut built_chunks: MessageWriter<ChunkBuilt>,
) {
    let mut completed: HashMap<ChunkPos, ChunkMesh> = HashMap::default();
    let mut warnings: HashMap<MesherWarning, u32> = HashMap::default();

    for (chunk_pos, build_task) in data.tasks.iter_mut() {
        if let Some(out) = future::block_on(future::poll_once(&mut build_task.0)) {
            completed.insert(*chunk_pos, out.0);

            for (warning, count) in out.1 {
                warnings
                    .entry(warning)
                    .and_modify(|e| *e += count)
                    .or_insert(count);
            }
        }
    }

    for pos in completed.keys() {
        data.tasks.remove(pos);
        built_chunks.write(ChunkBuilt(*pos));
    }

    data.chunk_meshes.extend(completed);

    log_warnings(warnings);
}

fn log_warnings(warnings: HashMap<MesherWarning, u32>) {
    if warnings.is_empty() {
        return;
    }

    let block_id_to_name: HashMap<BlockID, BlockTypeName> = iterate_over_block_registry()
        .map(|(name, block_id)| (*block_id, name.to_string()))
        .collect();

    for (warning, count) in warnings {
        let warn = match warning {
            MesherWarning::UnknownRenderShape(block_id) => {
                let block_type_name = block_id_to_name
                    .get(&block_id)
                    .cloned()
                    .unwrap_or("<unknown>".to_string());

                format!("{warning} ({block_type_name}) - occured {count} times")
            }
        };
        warn!(warn);
    }
}

fn debug_state(mut timer: ResMut<DebugTimer>, time: Res<Time>, data: Res<BuilderResources>) {
    timer.0.tick(time.delta());

    if timer.0.is_finished() {
        info!("{:?}", data);
    }
}
