use bevy::prelude::*;

use crate::bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial};

#[derive(Debug, Clone)]
pub enum MaterialHandle {
    PlaceHolder(Handle<StandardMaterial>),
    TexturedCube(Handle<TexturedCubeMaterial>),
    ColoredCube(Handle<ColoredCubeMaterial>),
    CutoutTexturedCube(Handle<CutoutTexturedCubeMaterial>),
}

impl MaterialHandle {
    pub fn spawn_entity(
        &self,
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        transform: Transform,
    ) -> Entity {
        match self {
            MaterialHandle::PlaceHolder(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::TexturedCube(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::ColoredCube(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
            MaterialHandle::CutoutTexturedCube(mat) => commands
                .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
                .id(),
        }
    }
}
