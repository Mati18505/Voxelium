use bevy::{asset::Asset, reflect::TypePath};
use shared::entities::{BlockID, BlockType, BlockTypeStorage};

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
struct BevyBlockTypeResource {
    name: String,
    affect_raycast: bool,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct BevyBlockTypeStorageResource {
    blocks: Vec<BevyBlockTypeResource>,
}

impl From<BevyBlockTypeStorageResource> for BlockTypeStorage {
    fn from(resource: BevyBlockTypeStorageResource) -> BlockTypeStorage {
        let block_types = resource
            .blocks
            .into_iter()
            .map(|e| BlockType::new(&e.name, e.affect_raycast))
            .collect();

        BlockTypeStorage::new(block_types)
    }
}

impl From<BevyBlockTypeStorageResource> for Vec<(String, BlockID)> {
    fn from(resource: BevyBlockTypeStorageResource) -> Self {
        resource
            .blocks
            .into_iter()
            .enumerate()
            .map(|(i, e)| (e.name, i as BlockID))
            .collect()
    }
}
