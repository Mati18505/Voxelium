use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    bevy_render::{ColoredCubeMaterial, TexturedCubeMaterial},
    chunk_mesh_builder::MaterialId,
};

#[derive(Debug)]
pub enum MaterialHandle {
    TexturedCube(Handle<TexturedCubeMaterial>),
    ColoredCube(Handle<ColoredCubeMaterial>),
}

impl MaterialHandle {
    pub fn spawn_entity(
        &self,
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        transform: Transform,
    ) -> Entity {
        match self {
            MaterialHandle::TexturedCube(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::ColoredCube(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
        }
    }
}

#[derive(Debug, Default)]
pub struct MaterialStorage {
    materials: HashMap<MaterialId, MaterialHandle>,
}

impl MaterialStorage {
    pub fn get(&self, id: MaterialId) -> Option<&MaterialHandle> {
        self.materials.get(&id)
    }

    pub fn add(&mut self, id: MaterialId, handle: MaterialHandle) -> &MaterialHandle {
        self.materials.insert(id, handle);
        self.get(id).unwrap()
    }
}
