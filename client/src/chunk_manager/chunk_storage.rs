use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::{hash_map, HashMap};

use shared::entities::{Chunk, ChunkPos, ChunkRepository};

use crate::chunk_mesh_builder::ChunkMesh;

#[derive(Message, Debug, Clone, PartialEq)]
pub struct SpawnChunk(pub ChunkPos, pub Chunk);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct DespawnChunk(pub ChunkPos);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct AddChunkMesh(pub ChunkPos, pub ChunkMesh);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct RemoveChunkMesh(pub ChunkPos);

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
        let entity = self.entity_map.entities.get(&pos)?;
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

#[derive(Component)]
pub struct ChunkMeshComponent(pub ChunkMesh);

pub struct ChunkStoragePlugin;
impl Plugin for ChunkStoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEntityMap>()
            .add_message::<SpawnChunk>()
            .add_message::<DespawnChunk>()
            .add_message::<AddChunkMesh>()
            .add_message::<RemoveChunkMesh>()
            .add_systems(
                Update,
                (
                    despawn_chunks,
                    spawn_chunks,
                    add_chunk_meshes,
                    remove_chunk_meshes,
                )
                    .chain(),
            );
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

fn add_chunk_meshes(
    mut meshes_to_add: MessageReader<AddChunkMesh>,
    mut commands: Commands,
    mut chunks: ResMut<ChunkEntityMap>,
    meshes: Query<&ChunkMeshComponent>,
) {
    for AddChunkMesh(pos, mesh) in meshes_to_add.read().cloned() {
        let mesh_component = ChunkMeshComponent(mesh);

        match chunks.entities.entry(pos) {
            hash_map::Entry::Occupied(entry) => {
                match meshes.get(*entry.get()) {
                    Ok(_) => warn!("Attempted to add chunk mesh which exists: {:?}", pos),
                    Err(_) => {
                        // If this unwrap panics, `ChunkEntityMap` had false data.
                        commands
                            .get_entity(*entry.get())
                            .unwrap()
                            .insert(mesh_component);
                    }
                };
            }
            hash_map::Entry::Vacant(_) => {
                warn!(
                    "Attempted to add chunk mesh to non-existent chunk: {:?}",
                    pos
                );
            }
        }
    }
}

fn remove_chunk_meshes(
    mut meshes_to_remove: MessageReader<RemoveChunkMesh>,
    mut commands: Commands,
    chunks: ResMut<ChunkEntityMap>,
) {
    for RemoveChunkMesh(pos) in meshes_to_remove.read() {
        match chunks.entities.get(pos) {
            Some(&entity) => {
                commands.entity(entity).remove::<ChunkMeshComponent>();
            }
            None => warn!("Attempted to remove non-existent chunk mesh: {:?}", pos),
        }
    }
}
