use std::{
    collections::{HashMap, HashSet},
    fmt,
    time::Duration,
};

use bevy::prelude::*;
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{
        iterate_over_block_registry, BlockID, ChunkPos, ChunkPosGenerator2D, ChunkPosGenerator3D,
        ChunkRepository,
    },
};

use crate::{
    bevy_resources::BlockTypeName,
    bevy_types::AppStates,
    chunk_manager::{ChunkMesherResource, ChunkStorage, ChunkUpdated, ControllerPos},
    chunk_mesh_builder::{
        meshers::{MesherWarning, MesherWarnings},
        ChunkMesh, ChunkMeshData,
    },
};

#[derive(Message, Debug, Clone, PartialEq)]
pub struct ChunkBuilt(pub ChunkPos, pub ChunkMesh);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct ChunkRemoved(pub ChunkPos);

#[derive(Resource, Debug, Clone)]
pub struct ChunkBuilderConfig {
    pub max_builds_per_frame: usize,
    pub render_distance: usize,
    pub dynamic_vertical_loading: bool,
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
            .add_message::<ChunkBuilt>()
            .add_message::<ChunkRemoved>()
            .add_systems(
                Update,
                (
                    update_desired_chunks,
                    rebuild_chunks,
                    build_chunks,
                    log_warnings,
                    debug_state,
                )
                    .run_if(in_state(AppStates::InGame)),
            );
    }
}

#[derive(Resource, Default)]
struct BuilderResources {
    pending_chunk_queue: PendingChunkQueue,
    built_chunks: HashSet<ChunkPos>,
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

fn update_desired_chunks(
    mut data: ResMut<BuilderResources>,
    mut removed_chunks: MessageWriter<ChunkRemoved>,
    config: Res<ChunkBuilderConfig>,
    player_pos: Res<ControllerPos>,
) {
    let generator: Box<dyn Iterator<Item = ChunkPos>> = match config.dynamic_vertical_loading {
        true => Box::new(ChunkPosGenerator3D::new(
            player_pos.0,
            config.render_distance,
        )),
        false => Box::new(ChunkPosGenerator2D::new(
            player_pos.0,
            config.render_distance,
        )),
    };
    let desired: HashSet<ChunkPos> = generator.collect();

    let to_add: Vec<ChunkPos> = desired
        .iter()
        .filter(|pos| {
            !data.built_chunks.contains(pos) && !data.pending_chunk_queue.contains_chunk(**pos)
        })
        .cloned()
        .collect();
    let to_remove: Vec<ChunkPos> = data
        .built_chunks
        .iter()
        .filter(|pos| !desired.contains(pos))
        .cloned()
        .collect();

    for pos in to_remove {
        data.pending_chunk_queue.remove_chunk(pos);
        data.built_chunks.remove(&pos);
        removed_chunks.write(ChunkRemoved(pos));
    }

    for pos in to_add {
        data.pending_chunk_queue.add_chunk(pos);
    }
}

fn rebuild_chunks(
    mut data: ResMut<BuilderResources>,
    mut chunks_updated: MessageReader<ChunkUpdated>,
) {
    for ChunkUpdated(pos, _chunk) in chunks_updated.read() {
        data.pending_chunk_queue.add_chunk(*pos);
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
        let Some(chunk) = chunks.0.get_chunk(chunk_pos) else {
            continue;
        };

        let mesher_result = mesher.0.create_mesh(chunk);
        let (layers, mesher_warnings) = (mesher_result.layers, mesher_result.warnings);
        let chunk_mesh = build_chunk_mesh(&layers);

        accum_mesher_warnings(&mut warnings, mesher_warnings);

        data.built_chunks.insert(chunk_pos);
        built_chunks.write(ChunkBuilt(chunk_pos, chunk_mesh));
    }
}

fn build_chunk_mesh(layers: &HashMap<u8, ChunkMeshData>) -> ChunkMesh {
    let mut chunk_mesh = ChunkMesh::default();

    for (material_id, mesh) in layers.iter() {
        let built = mesh.mesh().build();
        chunk_mesh.layers.insert(*material_id, built);
    }

    chunk_mesh
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

fn log_warnings(timer: Res<DebugTimer>, mut warnings: ResMut<MesherWarningsAccum>) {
    if !timer.0.is_finished() {
        return;
    }

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
