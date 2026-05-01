use bevy::{log::error, prelude::*};
use shared::entities::ChunkPos;

use crate::chunk_manager::{AddChunkMesh, RemoveChunkMesh};

use super::{material_handle::MaterialHandle, voxel_materials::RenderMaterials};

pub struct EntitiesManagerPlugin;
impl Plugin for EntitiesManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (despawn_chunk_mesh_layers, spawn_chunk_mesh_layers).chain());
    }
}

#[derive(Component, PartialEq, Clone, Copy)]
struct RelatedChunk(ChunkPos);

fn despawn_chunk_mesh_layers(
    mut meshes_to_remove: MessageReader<RemoveChunkMesh>,
    mut commands: Commands,
    q: Query<(Entity, &RelatedChunk)>
) {
    for RemoveChunkMesh(pos) in meshes_to_remove.read() {
        let desired_related_chunk = RelatedChunk(*pos);

        let entities_to_remove: Vec<Entity> = q.iter().filter(|(_, &related_chunk)| related_chunk == desired_related_chunk).map(|(entity, _)| entity).collect();

        for entity in entities_to_remove {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_chunk_mesh_layers(
    mut meshes_to_add: MessageReader<AddChunkMesh>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    render_materials: Res<RenderMaterials>,
) {
    for AddChunkMesh(pos, chunk_mesh) in meshes_to_add.read() {
        let transform = Transform::from_xyz(pos.x as f32, pos.y as f32, pos.z as f32);

        for (runtime_material_id, mesh) in chunk_mesh.layers() {
            let mesh = meshes.add(mesh.clone());

            if let Some(render_material) = render_materials.0.get(*runtime_material_id as usize) {
                spawn_chunk_mesh_layer(&mut commands, RelatedChunk(*pos), mesh, render_material, transform);
            } else {
                error!("Material {} not found!", runtime_material_id);
            }
        }
    }
}

fn spawn_chunk_mesh_layer(
    commands: &mut Commands,
    related_chunk: RelatedChunk,
    mesh: Handle<Mesh>,
    render_material: &MaterialHandle,
    transform: Transform,
) -> Entity {
    match render_material {
        MaterialHandle::Placeholder(mat) => commands.spawn((related_chunk, Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform)).id(),
        MaterialHandle::Textured(mat) => commands.spawn((related_chunk, Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform)).id(),
        MaterialHandle::Colored(mat) => commands.spawn((related_chunk, Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform)).id(),
        MaterialHandle::CutoutTextured(mat) => commands.spawn((related_chunk, Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform)).id(),
    }
}
