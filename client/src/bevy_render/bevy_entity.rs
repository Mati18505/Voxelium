use std::sync::Arc;

use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    },
    log,
    mesh::Mesh,
};

use super::BevyChunkMesh;
use crate::bevy_resources::MaterialStorage;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BevyChunkEntity {
    entities: Vec<Entity>,
}

impl BevyChunkEntity {
    pub fn new(
        chunk_mesh: BevyChunkMesh,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material_storage: Arc<MaterialStorage>,
    ) -> Self {
        let mut render_resource = BevyChunkEntity::default();

        for (material_id, mesh) in chunk_mesh.layers {
            let mesh_handle = meshes.add(mesh);

            if let Some(material) = material_storage.get_by_id(material_id as usize) {
                let entity = material.spawn_entity(commands, mesh_handle, chunk_mesh.transform);
                render_resource.entities.push(entity);
            } else {
                log::error!("Material {} not found!", material_id);
            }
        }

        render_resource
    }

    pub fn cleanup(&self, commands: &mut Commands) {
        for entity in self.entities.iter() {
            commands.entity(*entity).despawn();
        }
    }
}
