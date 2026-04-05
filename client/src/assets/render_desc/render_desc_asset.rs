use bevy::prelude::*;

use crate::bevy_resources::RenderDescDictionary;

#[derive(Debug, Asset, TypePath, Clone)]
pub struct RenderDescDictAsset(pub RenderDescDictionary);
