use std::sync::Arc;

use bevy::{ecs::resource::Resource, state::state::States};
use shared::entities::BlockTypeStorage;

use crate::bevy_resources::{MaterialStorage, RenderShapeStorage, TextureIdStorage};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AppStates {
    #[default]
    Loading,
    Compile,
    InGame,
}

#[derive(Resource)]
pub struct GameResources {
    pub server_block_type_storage: Arc<BlockTypeStorage>,
    pub texture_storage: Arc<TextureIdStorage>,
    pub material_storage: Arc<MaterialStorage>,
    pub render_shape_storage: Arc<RenderShapeStorage>,
}
