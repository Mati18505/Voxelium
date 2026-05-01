use bevy::prelude::*;

use super::materials::*;

#[derive(Debug, Clone)]
pub enum MaterialHandle {
    Placeholder(Handle<StandardMaterial>),
    Textured(Handle<TexturedCubeMaterial>),
    Colored(Handle<ColoredCubeMaterial>),
    CutoutTextured(Handle<CutoutTexturedCubeMaterial>),
}
