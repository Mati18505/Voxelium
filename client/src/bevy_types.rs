use std::sync::Arc;

use bevy::{asset::Handle, ecs::resource::Resource, image::Image, state::state::States};
use shared::entities::BlockTypeStorage;

use crate::bevy_resources::{MaterialStorage, RenderBlockTypeStorage, TextureIndexDictionary};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AppStates {
    #[default]
    Loading,
    InGame,
}

#[derive(Resource)]
pub struct GameResources {
    pub block_type_storage: Arc<RenderBlockTypeStorage>,
    pub server_block_type_storage: Arc<BlockTypeStorage>,
    pub texture_dictionary: Arc<TextureIndexDictionary>,
    pub material_storage: Arc<MaterialStorage>,
}
