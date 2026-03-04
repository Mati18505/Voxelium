use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial},
    chunk_mesh_builder::MaterialId,
};

#[derive(Debug)]
pub enum MaterialHandle {
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
