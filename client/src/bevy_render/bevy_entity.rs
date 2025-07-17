use bevy::{
    asset::{Assets, Handle},
    color::Color,
    ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    },
    image::Image,
    pbr::{MeshMaterial3d, StandardMaterial},
    render::mesh::{Mesh, Mesh3d},
    utils::default,
};

use super::bevy_voxel_render::VoxelMaterial;
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
        materials: &mut ResMut<Assets<VoxelMaterial>>,
        base_color_texture: Handle<Image>,
    ) -> Self {
        let mut render_resource = BevyChunkEntity::default();

        for (material_name, mesh) in chunk_mesh.layers {
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(VoxelMaterial {
                array_texture: base_color_texture.clone(),
            });
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    chunk_mesh.transform,
                ))
                .id();

            render_resource.entities.push(entity);
        }

        render_resource
    }

    pub fn cleanup(&self, commands: &mut Commands) {
        for entity in self.entities.iter() {
            commands.entity(*entity).despawn();
        }
    }
}
