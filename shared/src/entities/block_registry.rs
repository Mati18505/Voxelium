use super::BlockID;

pub trait BlockRegistry {
    fn name_to_block_id(&self, block_name: &str) -> BlockID;
}

pub trait IterableBlockRegistry {
    fn iter(&self) -> impl Iterator<Item = (&str, BlockID)>;
}
