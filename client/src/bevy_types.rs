use std::sync::Arc;

use bevy::{asset::Handle, ecs::resource::Resource, image::Image, state::state::States};

use crate::chunk_builder::{MeshBlockTypeStorage, TextureDictionary};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AppStates {
    #[default]
    Loading,
    InGame,
}

#[derive(Resource)]
pub struct GameResources {
    pub block_type_storage: Arc<MeshBlockTypeStorage>,
    pub server_block_type_storage: Arc<shared::entities::BlockTypeStorage>,
    pub texture_dictionary: Arc<TextureDictionary>, 
    pub opaque_texture: Handle<Image>,
}