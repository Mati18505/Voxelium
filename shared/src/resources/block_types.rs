use crate::entities::{BlockType, BlockTypeStorage};

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct BlockTypeResource {
    name: String,
    affect_raycast: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct BlockTypeStorageResource {
    blocks: Vec<BlockTypeResource>,
}

impl BlockTypeStorageResource {
    pub fn deserialize(content: &str) -> Result<BlockTypeStorageResource, serde_json::Error> {
        serde_json::from_str(content)
    }
}

impl From<BlockTypeStorageResource> for BlockTypeStorage {
    fn from(resource: BlockTypeStorageResource) -> Self {
        let block_types = resource
            .blocks
            .into_iter()
            .map(|e| BlockType::new(&e.name, e.affect_raycast))
            .collect();

        BlockTypeStorage::new(block_types)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let block_types = r#"
        {
            "blocks": [
            {
                "name": "air",
                "affect_raycast": false
            },
            {
                "name": "dirt",
                "affect_raycast": true
            },
            {
                "name": "stone",
                "affect_raycast": true
            }
            ]
        }
        "#;

        let resource = BlockTypeStorageResource::deserialize(block_types).unwrap();
        let storage: BlockTypeStorage = resource.into();

        let expected_block_types = vec![
            BlockType::new("air", false),
            BlockType::new("dirt", true),
            BlockType::new("stone", true),
        ];
        assert_eq!(storage, BlockTypeStorage::new(expected_block_types));
    }
}
