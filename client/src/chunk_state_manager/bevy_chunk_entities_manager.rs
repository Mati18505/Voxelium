use std::collections::HashMap;

use bevy::prelude::*;
use shared::entities::ChunkPos;

use crate::chunk_mesh_builder::ChunkMesh;
use crate::bevy_render::{BevyChunkEntity, BevyChunkMesh, VoxelMaterial};
use super::chunk_state_manager;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChunkEntitiesManager {
    pending_to_create: HashMap<ChunkPos, ChunkMesh>,
    pending_to_remove: Vec<ChunkPos>,
    chunk_entities: HashMap<ChunkPos, BevyChunkEntity>,
}

impl chunk_state_manager::ChunkObjectCallback for ChunkEntitiesManager {
    fn chunk_object_created(&mut self, chunk_pos: ChunkPos, chunk_mesh: &ChunkMesh) {
        self.pending_to_create.insert(chunk_pos, chunk_mesh.clone());
    }
    fn chunk_object_removed(&mut self, chunk_pos: ChunkPos) {
        self.pending_to_remove.push(chunk_pos);
    }
}

impl ChunkEntitiesManager {
    pub fn process_pending(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        opaque_texture: Handle<Image>,
        voxel_materials: &mut ResMut<Assets<VoxelMaterial>>,
    ) {
        for (pos, mesh) in std::mem::take(&mut self.pending_to_create) {
            self.remove_chunk_entity(&pos, commands);
            self.create_chunk_entity(pos, mesh, commands, meshes, opaque_texture.clone(), voxel_materials);
        }

        for pos in std::mem::take(&mut self.pending_to_remove) {
            self.remove_chunk_entity(&pos, commands);
        }

        assert!(self.pending_to_create.len() == 0);
        assert!(self.pending_to_remove.len() == 0);
    }

    fn create_chunk_entity(
        &mut self, 
        pos: ChunkPos,
        mesh: ChunkMesh,
        mut commands: &mut Commands,
        mut meshes: &mut ResMut<Assets<Mesh>>,
        opaque_texture: Handle<Image>,
        mut voxel_materials: &mut ResMut<Assets<VoxelMaterial>>,
    ) {
        assert!(self.chunk_entities.get(&pos).is_none(), "Potential memory leak!");

        let mut mesh = BevyChunkMesh::from(mesh);
        mesh.apply_transform(Transform::from_xyz(pos.x as f32, pos.y as f32, pos.z as f32));

        let chunk_entity = BevyChunkEntity::new(
            mesh,
            &mut commands,
            &mut meshes,
            &mut voxel_materials,
            opaque_texture,
        );

        self.chunk_entities.insert(pos, chunk_entity);
    }

    fn remove_chunk_entity(
        &mut self,
        pos: &ChunkPos,
        mut commands: &mut Commands,
    ) {
        if let Some(entity) = self.chunk_entities.get(pos) {
            entity.cleanup(&mut commands);
            self.chunk_entities.remove(pos);
        }
    }
}