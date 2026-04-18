use bevy::prelude::*;

use crate::{bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial}, bevy_resources::{RuntimeMaterial, TextureIdStorage}};

#[derive(Debug, Clone)]
pub enum MaterialHandle {
    Placeholder(Handle<StandardMaterial>),
    Textured(Handle<TexturedCubeMaterial>),
    Colored(Handle<ColoredCubeMaterial>),
    CutoutTextured(Handle<CutoutTexturedCubeMaterial>),
}

impl MaterialHandle {
    pub fn spawn_entity(
        &self,
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        transform: Transform,
    ) -> Entity {
        match self {
            MaterialHandle::Placeholder(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::Textured(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::Colored(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::CutoutTextured(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
        }
    }
}
