use crate::types::CHUNK_SIZE;
use crate::block_storage::BlockStorage;

#[derive(Debug)]
pub struct Chunk {
    block_storage: BlockStorage,
}

impl Chunk {
    pub fn new(block_storage: BlockStorage) -> Self {
        Chunk {
            block_storage: block_storage
        }
    }
    pub fn get_block_storage_mut(&mut self) -> &mut BlockStorage {
        return &mut self.block_storage;
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