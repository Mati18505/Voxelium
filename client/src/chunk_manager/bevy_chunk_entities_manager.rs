use std::collections::HashMap;

use bevy::prelude::*;
use shared::entities::ChunkPos;

use crate::bevy_render::BevyChunkEntity;
use crate::bevy_types::{AppStates, GameResources};
use crate::chunk_mesh_builder::ChunkMesh;

#[derive(Message, Debug, Clone, PartialEq)]
pub struct CreateEntity(pub ChunkPos, pub ChunkMesh);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct RemoveEntity(pub ChunkPos);

pub struct ChunkEntitiesPlugin;
impl Plugin for ChunkEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<CreateEntity>()
            .add_message::<RemoveEntity>()
            .init_resource::<ChunkEntitiesResource>().add_systems(
            Update,
            (process_create_requests, process_remove_requests).run_if(in_state(AppStates::InGame)),
        );
    }
}

#[derive(Resource, Debug, Default)]
struct ChunkEntitiesResource {
    chunk_entities: HashMap<ChunkPos, BevyChunkEntity>,
}

fn process_create_requests(
    mut entity_create_requests: MessageReader<CreateEntity>,
    mut data: ResMut<ChunkEntitiesResource>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_resources: Res<GameResources>,
) {
    for request in entity_create_requests.read() {
        let pos = request.0;
        let mesh: ChunkMesh = request.1.clone();

        if let Some(entity) = data.chunk_entities.get(&pos) {
            entity.cleanup(&mut commands);
            data.chunk_entities.remove(&pos);
        }

        let transform = Transform::from_xyz(pos.x as f32, pos.y as f32, pos.z as f32);
        let chunk_entity = BevyChunkEntity::new(
            mesh,
            &mut commands,
            &mut meshes,
            &game_resources.material_storage,
            transform,
        );

        data.chunk_entities.insert(pos, chunk_entity);
    }
}

fn process_remove_requests(
    mut entity_remove_requests: MessageReader<RemoveEntity>,
    mut data: ResMut<ChunkEntitiesResource>,
    mut commands: Commands,
) {
    for request in entity_remove_requests.read() {
        let pos = request.0;

        if let Some(entity) = data.chunk_entities.get(&pos) {
            entity.cleanup(&mut commands);
            data.chunk_entities.remove(&pos);
        }
    }
}
