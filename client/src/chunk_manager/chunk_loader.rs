use bevy::prelude::*;
use shared::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{Chunk, ChunkPos, ChunkPosGenerator2D},
};
use std::{collections::HashSet, fmt, time::Duration};

use crate::{
    bevy_types::AppStates,
    chunk_manager::{ChunkProviderResource, ControllerPos},
};

#[derive(Message, Debug, Clone, PartialEq)]
pub struct ChunkLoaded(pub ChunkPos, pub Chunk);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct ChunkUnloaded(pub ChunkPos);

#[derive(Resource, Debug, Clone)]
pub struct ChunkLoaderConfig {
    pub max_loads_per_frame: usize,
    pub load_distance: usize,
    pub dynamic_vertical_loading: bool,
    pub debug: bool,
}

pub struct ChunkLoaderPlugin(ChunkLoaderConfig);
impl ChunkLoaderPlugin {
    pub fn new(config: ChunkLoaderConfig) -> Self {
        Self(config)
    }
}
impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .init_resource::<LoaderResources>()
            .insert_resource(DebugTimer(Timer::new(
                Duration::from_secs(2),
                TimerMode::Repeating,
            )))
            .add_message::<ChunkLoaded>()
            .add_message::<ChunkUnloaded>()
            .add_systems(
                Update,
                (
                    update_desired_chunks.run_if(resource_changed::<ControllerPos>),
                    load_chunks,
                    debug_state,
                )
                    .run_if(in_state(AppStates::InGame)),
            );
    }
}

#[derive(Resource, Default)]
struct LoaderResources {
    pending_chunk_queue: PendingChunkQueue,
    loaded_chunks: HashSet<ChunkPos>,
    desired_chunks: HashSet<ChunkPos>,
}

impl fmt::Debug for LoaderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoaderResources")
            .field("pending_chunk_queue", &self.pending_chunk_queue)
            .field("loaded_chunks", &self.loaded_chunks.len())
            .field("desired_chunks", &self.desired_chunks.len())
            .finish()
    }
}

#[derive(Resource)]
struct DebugTimer(Timer);

fn update_desired_chunks(
    mut data: ResMut<LoaderResources>,
    mut unloaded_chunks: MessageWriter<ChunkUnloaded>,
    config: Res<ChunkLoaderConfig>,
    player_pos: Res<ControllerPos>,
) {
    let generator: Box<dyn Iterator<Item = ChunkPos>> = match config.dynamic_vertical_loading {
        true => Box::new(shared::entities::ChunkPosGenerator3D::new(
            player_pos.0,
            config.load_distance,
        )),
        false => Box::new(ChunkPosGenerator2D::new(player_pos.0, config.load_distance)),
    };
    let desired: HashSet<ChunkPos> = generator.collect();

    let to_add: Vec<ChunkPos> = desired
        .iter()
        .filter(|pos| {
            !data.loaded_chunks.contains(pos) && !data.pending_chunk_queue.contains_chunk(**pos)
        })
        .cloned()
        .collect();
    let to_remove: Vec<ChunkPos> = data
        .loaded_chunks
        .iter()
        .filter(|pos| !desired.contains(pos))
        .cloned()
        .collect();

    for pos in to_remove {
        data.pending_chunk_queue.remove_chunk(pos);
        data.loaded_chunks.remove(&pos);
        unloaded_chunks.write(ChunkUnloaded(pos));
    }

    for pos in to_add {
        data.pending_chunk_queue.add_chunk(pos);
    }

    data.desired_chunks = desired;
}

fn load_chunks(
    mut data: ResMut<LoaderResources>,
    mut loaded_chunks: MessageWriter<ChunkLoaded>,
    mut chunk_provider: ResMut<ChunkProviderResource>,
    player_pos: Res<ControllerPos>,
    config: Res<ChunkLoaderConfig>,
) {
    for chunk_pos in data
        .pending_chunk_queue
        .take_nearest_chunks(config.max_loads_per_frame, player_pos.0)
    {
        if !data.desired_chunks.contains(&chunk_pos) {
            continue;
        }

        let chunk = chunk_provider.0.load_chunk(chunk_pos);

        data.loaded_chunks.insert(chunk_pos);
        loaded_chunks.write(ChunkLoaded(chunk_pos, chunk));
    }
}

fn debug_state(
    mut timer: ResMut<DebugTimer>,
    time: Res<Time>,
    data: Res<LoaderResources>,
    config: Res<ChunkLoaderConfig>,
) {
    if !config.debug {
        return;
    }

    timer.0.tick(time.delta());

    if timer.0.is_finished() {
        info!("{:?}", data);
    }
}
