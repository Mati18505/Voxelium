use std::sync::Arc;

use bevy::{asset::Handle, ecs::resource::Resource, image::Image, state::state::States};
use shared::entities::BlockTypeStorage;

use crate::{bevy_resources::{MaterialStorage, RenderDescDictionary, RenderShapeStorage, TextureIndexDictionary}};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AppStates {
    #[default]
    Loading,
    Compile,
    InGame,
}

#[derive(Resource)]
pub struct GameResources {
    pub render_desc_dict: Arc<RenderDescDictionary>,
    pub server_block_type_storage: Arc<BlockTypeStorage>,
    pub texture_index_dictionary: Arc<TextureIndexDictionary>,
    pub material_storage: Arc<MaterialStorage>,
    pub render_shape_storage: Arc<RenderShapeStorage>,
}
