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

impl Into<BlockTypeStorage> for BevyBlockTypeStorageResource {
    fn into(self) -> BlockTypeStorage {
        let block_types = self
            .blocks
            .into_iter()
            .map(|e| BlockType::new(&e.name, e.affect_raycast))
            .collect();

        BlockTypeStorage::new(block_types)
    }
}

impl Into<Vec<(String, BlockID)>> for BevyBlockTypeStorageResource {
    fn into(self) -> Vec<(String, BlockID)> {
        self
            .blocks
            .into_iter()
            .map(|e| e.name)
            .enumerate()
            .map(|(id, name)| (name, id as BlockID))
            .collect()
    }
}