use std::sync::Arc;

use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    },
    image::Image,
    log,
    mesh::Mesh,
    pbr::MeshMaterial3d,
    render::mesh::{Mesh, Mesh3d},
    transform::components::Transform,
};

use super::bevy_voxel_render::VoxelMaterial;
use super::BevyChunkMesh;
use crate::{bevy_resources::MaterialStorage, chunk_mesh_builder::ChunkMesh};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BevyChunkEntity {
    entities: Vec<Entity>,
}

impl BevyChunkEntity {
    pub fn new(
        chunk_mesh: ChunkMesh,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<VoxelMaterial>>,
        base_color_texture: Handle<Image>,
        material_storage: Arc<MaterialStorage>,
        transform: Transform,
    ) -> Self {
        let mut render_resource = BevyChunkEntity::default();

        for (material_id, mesh) in chunk_mesh.layers {
            let mesh_handle = meshes.add(mesh);

            render_resource.entities.push(entity);

            if let Some(material) = material_storage.get_by_id(material_id as usize) {
                let entity = material.spawn_entity(commands, mesh_handle, transform);
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
