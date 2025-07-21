use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    bevy_render::{BevyChunkEntity, BevyChunkMesh, VoxelMaterial},
    bevy_types::AppStates,
    chunk_manager::{chunk_entities_manager, events::*},
    chunk_mesh_builder::ChunkMesh,
};
use shared::entities::ChunkPos;
pub struct ChunkEntitiesManagerPlugin;
impl Plugin for ChunkEntitiesManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChunkEntityEvent>()
            .insert_resource(ChunkEntitiesManager::default())
            .add_systems(Update, entities_manager.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Resource, Debug)]
pub struct ChunkEntitiesManagerResource {
    opaque_texture: Handle<Image>,
}

impl ChunkEntitiesManagerResource {
    pub fn new(opaque_texture: Handle<Image>) -> Self {
        Self { opaque_texture }
    }
}

#[derive(Resource, Debug, Default)]
struct ChunkEntitiesManager {
    chunk_entities: HashMap<ChunkPos, BevyChunkEntity>,
}

fn entities_manager(
    mut chunk_entities_manager: ResMut<ChunkEntitiesManager>,
    mut chunk_entities_req_ev: EventReader<ChunkEntityEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
    chunk_entities_manager_res: Res<ChunkEntitiesManagerResource>,
) {
    for ev in chunk_entities_req_ev.read() {
        match ev {
            ChunkEntityEvent::Create(chunk_pos, chunk_mesh) => {
                remove_chunk_entity(&mut chunk_entities_manager, &chunk_pos, &mut commands);
                create_chunk_entity(
                    &mut chunk_entities_manager,
                    *chunk_pos,
                    chunk_mesh.clone(),
                    &mut commands,
                    &mut meshes,
                    &mut voxel_materials,
                    &chunk_entities_manager_res,
                );
            }
            ChunkEntityEvent::Remove(chunk_pos) => {
                remove_chunk_entity(&mut chunk_entities_manager, &chunk_pos, &mut commands);
            }
        }
    }
}

fn create_chunk_entity(
    mut chunk_entities_manager: &mut ResMut<ChunkEntitiesManager>,
    pos: ChunkPos,
    mesh: ChunkMesh,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    voxel_materials: &mut ResMut<Assets<VoxelMaterial>>,
    chunk_entities_manager_res: &Res<ChunkEntitiesManagerResource>,
) {
    assert!(
        !chunk_entities_manager.chunk_entities.contains_key(&pos),
        "Potential memory leak!"
    );

    let mut mesh = BevyChunkMesh::from(mesh);
    mesh.apply_transform(Transform::from_xyz(
        pos.x as f32,
        pos.y as f32,
        pos.z as f32,
    ));

    let chunk_entity = BevyChunkEntity::new(
        mesh,
        commands,
        meshes,
        voxel_materials,
        chunk_entities_manager_res.opaque_texture.clone(),
    );

    chunk_entities_manager
        .chunk_entities
        .insert(pos, chunk_entity);
}

fn remove_chunk_entity(
    mut chunk_entities_manager: &mut ResMut<ChunkEntitiesManager>,
    pos: &ChunkPos,
    commands: &mut Commands,
) {
    if let Some(entity) = chunk_entities_manager.chunk_entities.get(pos) {
        entity.cleanup(commands);
        chunk_entities_manager.chunk_entities.remove(pos);
    }
}
