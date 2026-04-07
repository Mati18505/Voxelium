use std::{
    collections::{HashMap, HashSet},
    fmt,
    time::Duration,
};

use bevy::prelude::*;
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{
        BlockID, BlockRegistry, BlockSide, Chunk, ChunkPos, ChunkPosGenerator2D,
        ChunkPosGenerator3D, ChunkRepository, Direction, IterableBlockRegistry, CHUNK_SIZE,
    },
};

use crate::{
    bevy_resources::{BlockNameToId, BlockTypeName},
    bevy_types::AppStates,
    chunk_manager::{ChunkMesherResource, ChunkStorage, ChunkUpdated, ControllerPos},
    chunk_mesh_builder::{
        meshers::{MesherWarning, MesherWarnings},
        ChunkMesh, ChunkMeshData, ChunkWithNeighbors,
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
    pub debug: bool,
    // Used only with 2d generator.
    pub height: usize,
}

impl Default for ChunkBuilderConfig {
    fn default() -> Self {
        Self {
            max_builds_per_frame: 16,
            render_distance: 8,
            dynamic_vertical_loading: false,
            debug: false,
            height: 1,
        }
    }
}

pub struct ChunkBuilderPlugin;
impl Plugin for ChunkBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkBuilderConfig>()
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
                    update_desired_chunks.run_if(resource_changed::<ControllerPos>),
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
    desired_chunks: HashSet<ChunkPos>,
}

impl fmt::Debug for BuilderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuilderResources")
            .field("pending_chunk_queue", &self.pending_chunk_queue)
            .field("built_chunks", &self.built_chunks.len())
            .field("desired_chunks", &self.desired_chunks.len())
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
        true => Box::new(shared::entities::ChunkPosGenerator3D::new(
            player_pos.0,
            config.render_distance,
        )),
        false => Box::new(
            ChunkPosGenerator2D::new(player_pos.0, config.render_distance).flat_map(|pos| {
                let y_iter = (0..config.height).map(|e| e * 16);
                y_iter.map(move |y| ChunkPos::new(pos.x, y as isize, pos.z))
            }),
        ),
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
        .chain(data.pending_chunk_queue.iter())
        .filter(|pos| !desired.contains(pos))
        .cloned()
        .collect();

    for pos in to_remove {
        data.pending_chunk_queue.remove_chunk(pos);
        if data.built_chunks.remove(&pos) {
            removed_chunks.write(ChunkRemoved(pos));
        }
    }

    for pos in to_add {
        data.pending_chunk_queue.add_chunk(pos);
    }

    data.desired_chunks = desired;
}

fn rebuild_chunks(
    mut data: ResMut<BuilderResources>,
    mut chunks_updated: MessageReader<ChunkUpdated>,
) {
    for ChunkUpdated(pos, _chunk) in chunks_updated.read() {
        data.pending_chunk_queue.add_chunk(*pos);

        for pos in iter_neighbors(*pos) {
            data.pending_chunk_queue.add_chunk(pos);
        }
    }
}

fn iter_neighbors(pos: ChunkPos) -> impl Iterator<Item = ChunkPos> {
    BlockSide::iterator().map(move |side| {
        let dir = Direction::from(*side);
        let dif = dir * CHUNK_SIZE as isize;

        ChunkPos::new(pos.x + dif.x, pos.y + dif.y, pos.z + dif.z)
    })
}

fn build_chunks(
    mut data: ResMut<BuilderResources>,
    mut built_chunks: MessageWriter<ChunkBuilt>,
    mut warnings: ResMut<MesherWarningsAccum>,
    chunks: ChunkStorage,
    mesher: Res<ChunkMesherResource>,
    player_pos: Res<ControllerPos>,
    config: Res<ChunkBuilderConfig>,
) {
    for chunk_pos in data
        .pending_chunk_queue
        .take_nearest_chunks(config.max_builds_per_frame, player_pos.0)
    {
        if !data.desired_chunks.contains(&chunk_pos) {
            continue;
        }

        let Some(chunk_with_neighbors) = create_chunk_with_neighbors(chunk_pos, &chunks) else {
            data.pending_chunk_queue.add_chunk(chunk_pos);
            continue;
        };

        let mesher_result = mesher.0.create_mesh(&chunk_with_neighbors);
        let (layers, mesher_warnings) = (mesher_result.layers, mesher_result.warnings);
        let chunk_mesh = build_chunk_mesh(&layers);

        accum_mesher_warnings(&mut warnings, mesher_warnings);

        data.built_chunks.insert(chunk_pos);
        built_chunks.write(ChunkBuilt(chunk_pos, chunk_mesh));
    }
}

/// Creates `ChunkWithNeighbors` if chunk and all neighbors are loaded, else returns None.
fn create_chunk_with_neighbors<'a>(
    origin_pos: ChunkPos,
    chunks: &'a ChunkStorage,
) -> Option<ChunkWithNeighbors<'a>> {
    let origin_chunk = chunks.get_chunk(origin_pos)?;

    let mut iter = iter_neighbors(origin_pos);

    let neighbors: [Option<&Chunk>; 6] = std::array::from_fn(|_| {
        let pos = iter.next().unwrap();
        chunks.get_chunk(pos)
    });

    for side in BlockSide::iterator() {
        if *side != BlockSide::Top
            && *side != BlockSide::Bottom
            && neighbors[*side as usize].is_none()
        {
            return None;
        }
    }

    Some(ChunkWithNeighbors {
        chunk: origin_chunk,
        neighbors,
    })
}

fn build_chunk_mesh(layers: &HashMap<u8, ChunkMeshData>) -> ChunkMesh {
    let mut chunk_mesh = ChunkMesh::default();

    for (material_id, mesh) in layers.iter() {
        let built = mesh.mesh().build();
        chunk_mesh.add_layer(*material_id, built);
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

fn log_warnings(
    timer: Res<DebugTimer>,
    mut warnings: ResMut<MesherWarningsAccum>,
    registry: Res<BlockNameToId>,
) {
    if !timer.0.is_finished() {
        return;
    }

    let warnings = std::mem::take(&mut warnings.0);

    if warnings.is_empty() {
        return;
    }

    let block_id_to_name: HashMap<BlockID, BlockTypeName> = registry
        .iter()
        .map(|(name, block_id)| (block_id, name.to_string()))
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

fn debug_state(
    mut timer: ResMut<DebugTimer>,
    time: Res<Time>,
    data: Res<BuilderResources>,
    config: Res<ChunkBuilderConfig>,
) {
    if !config.debug {
        return;
    }

    timer.0.tick(time.delta());

    if timer.0.is_finished() {
        info!("{:?}", data);
    }
}
