use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::{hash_map, HashMap};

use shared::entities::{Chunk, ChunkPos, ChunkRepository};

#[derive(Message, Debug, Clone, PartialEq)]
pub struct SpawnChunk(pub ChunkPos, pub Chunk);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct DespawnChunk(pub ChunkPos);

#[derive(SystemParam)]
pub struct ChunkStorage<'w, 's> {
    entity_map: Res<'w, ChunkEntityMap>,
    chunks: Query<'w, 's, &'static mut ChunkComponent>,
}

impl<'w, 's> ChunkRepository for ChunkStorage<'w, 's> {
    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        let entity = self.entity_map.entities.get(&pos)?;
        self.chunks.get(*entity).ok().map(|c| &c.0)
    }
    fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        let entity = self.entity_map.entities.get(&pos).unwrap();
        self.chunks
            .get_mut(*entity)
            .ok()
            .map(|c| &mut c.into_inner().0)
    }
}

#[derive(Component)]
pub struct ChunkPosComponent(pub ChunkPos);

#[derive(Component)]
pub struct ChunkComponent(pub Chunk);

pub struct ChunkStoragePlugin;
impl Plugin for ChunkStoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEntityMap>()
            .add_message::<SpawnChunk>()
            .add_message::<DespawnChunk>()
            .add_systems(Update, spawn_chunks.after(despawn_chunks));
    }
}

#[derive(Resource, Default)]
struct ChunkEntityMap {
    entities: HashMap<ChunkPos, Entity>,
}

fn spawn_chunks(
    mut chunks_to_spawn: MessageReader<SpawnChunk>,
    mut commands: Commands,
    mut chunks: ResMut<ChunkEntityMap>,
) {
    for to_spawn in chunks_to_spawn.read() {
        let SpawnChunk(pos, chunk) = to_spawn.clone();
        let pos_component = ChunkPosComponent(pos);
        let chunk_component = ChunkComponent(chunk);

        if let hash_map::Entry::Vacant(entry) = chunks.entities.entry(pos) {
            let entity = commands.spawn((pos_component, chunk_component)).id();
            entry.insert(entity);
        } else {
            warn!("Attempted to spawn chunk which exists: {:?}", pos);
        }
    }
}

fn despawn_chunks(
    mut chunks_to_despawn: MessageReader<DespawnChunk>,
    mut commands: Commands,
    mut chunks: ResMut<ChunkEntityMap>,
) {
    for to_despawn in chunks_to_despawn.read() {
        let DespawnChunk(pos) = to_despawn;

        match chunks.entities.remove(pos) {
            Some(entity) => commands.entity(entity).despawn(),
            None => warn!("Attempted to despawn non-existent chunk: {:?}", pos),
        }
    }
}
