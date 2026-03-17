use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use shared::entities::ChunkPos;

use super::ChunkObjectEvent;
use crate::bevy_render::BevyChunkEntity;
use crate::bevy_resources::MaterialStorage;
use crate::chunk_mesh_builder::ChunkMesh;

#[derive(Debug, Clone)]
pub struct ChunkEntitiesManager {
    rx: crossbeam_channel::Receiver<ChunkObjectEvent>,
    chunk_entities: HashMap<ChunkPos, BevyChunkEntity>,
}

impl ChunkEntitiesManager {
    pub fn new(rx: crossbeam_channel::Receiver<ChunkObjectEvent>) -> Self {
        Self {
            rx,
            chunk_entities: HashMap::new(),
        }
    }
    pub fn process_pending(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material_storage: Arc<MaterialStorage>,
    ) {
        while let Ok(chunk_obj_ev) = self.rx.try_recv() {
            match chunk_obj_ev {
                ChunkObjectEvent::Created(chunk_pos, chunk_mesh) => {
                    self.remove_chunk_entity(&chunk_pos, commands);
                    self.create_chunk_entity(
                        chunk_pos,
                        chunk_mesh,
                        commands,
                        meshes,
                        material_storage.clone(),
                    );
                }
                ChunkObjectEvent::Removed(chunk_pos) => {
                    self.remove_chunk_entity(&chunk_pos, commands);
                }
            }
        }
    }

    fn create_chunk_entity(
        &mut self,
        pos: ChunkPos,
        mesh: ChunkMesh,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material_storage: Arc<MaterialStorage>,
    ) {
        assert!(
            !self.chunk_entities.contains_key(&pos),
            "Potential memory leak!"
        );

        let transform = Transform::from_xyz(
            pos.x as f32,
            pos.y as f32,
            pos.z as f32,
        );

        let chunk_entity = BevyChunkEntity::new(mesh, commands, meshes, material_storage, transform);

        self.chunk_entities.insert(pos, chunk_entity);
    }

    fn remove_chunk_entity(&mut self, pos: &ChunkPos, commands: &mut Commands) {
        if let Some(entity) = self.chunk_entities.get(pos) {
            entity.cleanup(commands);
            self.chunk_entities.remove(pos);
        }
    }
}
