use super::block_storage::BlockStorage;

#[derive(Debug, Default, Clone, PartialEq)]
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