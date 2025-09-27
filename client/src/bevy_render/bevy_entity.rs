use std::sync::Arc;

use bevy::{
    asset::{Assets, Handle}, color, ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    }, image::Image, log, pbr::MeshMaterial3d, render::mesh::{Mesh, Mesh3d}
};

use crate::bevy_resources::{MaterialHandle, MaterialStorage};
use super::BevyChunkMesh;

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

            if let Some(material) = material_storage.get(material_id) {
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
