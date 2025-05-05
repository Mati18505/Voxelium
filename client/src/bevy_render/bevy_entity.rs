use bevy::{
    asset::{Assets, Handle},
    color::Color,
    ecs::{
        entity::Entity,
        system::{Commands, ResMut},
    },
    math::{Quat, Vec3},
    pbr::{MeshMaterial3d, StandardMaterial},
    render::mesh::{Mesh, Mesh3d},
    transform::components::Transform,
    utils::default,
};

use super::BevyChunkMesh;

#[derive(Debug, Default, Clone)]
pub struct BevyChunkEntity {
    entities: Vec<Entity>,
}

impl BevyChunkEntity {
    pub fn new(
        chunk_mesh: BevyChunkMesh,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let mut render_resource = BevyChunkEntity::default();

        for (material_name, mesh) in chunk_mesh.layers {
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgb(0.396, 0.263, 0.129),
                ..default()
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
}
