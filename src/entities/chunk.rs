use super::block_storage::BlockStorage;

#[derive(Debug, PartialEq)]
pub struct Chunk {
    block_storage: BlockStorage,
}

impl Chunk {
    pub fn new(block_storage: BlockStorage) -> Self {
        Chunk {
            block_storage
        }
    }

    pub fn get_block_storage(&self) -> &BlockStorage {
        return &self.block_storage;
    }
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Chunk {
            block_storage: self.block_storage.clone()
        }
    }
} 

