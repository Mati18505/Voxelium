use crate::entities::CHUNK_SIZE;

use super::block_storage::BlockStorage;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    block_storage: BlockStorage,
}

impl Chunk {
    pub fn new(block_storage: BlockStorage) -> Self {
        assert!(block_storage.iter().len() == CHUNK_SIZE.pow(3));

        Chunk { block_storage }
    }

    pub fn get_block_storage(&self) -> &BlockStorage {
        &self.block_storage
    }
}

impl Default for Chunk {
    fn default() -> Self {
        let block_storage = BlockStorage::new(vec![0; CHUNK_SIZE.pow(3)]);
        Self::new(block_storage)
    }
}
