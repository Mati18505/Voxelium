use std::{collections::HashMap, fmt, time::Duration};

use bevy::prelude::*;
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{iterate_over_block_registry, BlockID, ChunkPos, ChunkRepository},
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
pub struct ChunkBuilt(pub ChunkPos, pub ChunkMesh);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct ChunkRemoved(pub ChunkPos);

#[derive(Resource, Debug, Clone)]
pub struct ChunkBuilderConfig {
    pub max_builds_per_frame: usize,
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
            .init_resource::<MesherWarningsAccum>()
            .insert_resource(DebugTimer(Timer::new(
                Duration::from_secs(2),
                TimerMode::Repeating,
            )))
            .add_message::<BuildChunk>()
            .add_message::<RemoveChunk>()
            .add_message::<ChunkBuilt>()
            .add_message::<ChunkRemoved>()
            .add_systems(
                Update,
                (
                    (process_chunks_to_build, process_chunks_to_remove).chain(),
                    (build_chunks, log_warnings).chain(),
                    debug_state,
                )
                    .run_if(in_state(AppStates::InGame)),
            );
    }
}

#[derive(Resource, Default)]
struct BuilderResources {
    pending_chunk_queue: PendingChunkQueue,
}

impl fmt::Debug for BuilderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncChunkBuilder")
            .field("pending_chunk_queue", &self.pending_chunk_queue)
            .finish()
    }
}

#[derive(Resource)]
struct DebugTimer(Timer);

fn process_chunks_to_build(
    mut reader: MessageReader<BuildChunk>,
    mut data: ResMut<BuilderResources>,
) {
    for message in reader.read() {
        let chunk_pos = message.0;

        data.pending_chunk_queue.add_chunk(chunk_pos);
    }
}

fn process_chunks_to_remove(
    mut reader: MessageReader<RemoveChunk>,
    mut removed: MessageWriter<ChunkRemoved>,
    mut data: ResMut<BuilderResources>,
) {
    for message in reader.read() {
        let chunk_pos = message.0;

        data.pending_chunk_queue.remove_chunk(chunk_pos);
        removed.write(ChunkRemoved(chunk_pos));
    }
}

fn build_chunks(
    mut data: ResMut<BuilderResources>,
    mut built_chunks: MessageWriter<ChunkBuilt>,
    mut warnings: ResMut<MesherWarningsAccum>,
    chunks: Res<ChunkStorage>,
    mesher: Res<ChunkMesherResource>,
    player_pos: Res<ControllerPos>,
    config: Res<ChunkBuilderConfig>,
) {
    for chunk_pos in data
        .pending_chunk_queue
        .take_nearest_chunks(config.max_builds_per_frame, player_pos.0)
    {
        let Some(chunk) = chunks.get_chunk(chunk_pos) else {
            continue;
        };

        let mesher_result = mesher.0.create_mesh(chunk);
        let (layers, mesher_warnings) = (mesher_result.layers, mesher_result.warnings);
        let mut chunk_mesh: ChunkMesh = Default::default();

        for (material_id, mesh) in layers.iter() {
            let built = mesh.mesh().build();
            chunk_mesh.layers.insert(*material_id, built);
        }

        accum_mesher_warnings(&mut warnings, mesher_warnings);

        built_chunks.write(ChunkBuilt(chunk_pos, chunk_mesh));
    }
}

#[derive(Resource, Default)]
struct MesherWarningsAccum(HashMap<MesherWarning, u32>);

fn accum_mesher_warnings(accum: &mut MesherWarningsAccum, warnings: MesherWarnings) {
    for (warning, count) in warnings {
        accum
            .0
            .entry(warning)
            .and_modify(|e| *e += count)
            .or_insert(count);
    }
}

fn log_warnings(mut warnings: ResMut<MesherWarningsAccum>) {
    let warnings = std::mem::take(&mut warnings.0);

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
