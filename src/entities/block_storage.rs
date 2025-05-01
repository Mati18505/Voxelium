use cgmath::Vector3;
use crate::shared;
use std::fmt;

use super::{types::{BlockID, CHUNK_SIZE}, BlockInChunkPos};

#[derive(Clone, PartialEq)]
pub struct BlockStorage {
    block_types: Vec<BlockID>,
}

impl fmt::Debug for BlockStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const COLS: i32 = 25;
        let mut out: String = "".to_string();
        let mut counter = 0;

        for i in self.block_types.iter() {
            if counter == 0 {
                out += "\r\n";
            }

            out += &i.to_string();

            counter += 1;
            counter = counter % COLS;
        } 

        write!(f, "{}", out).unwrap();
        Ok(())
    }
}

impl BlockStorage {
    pub fn new(blocks: Vec<BlockID>) -> Self {
        BlockStorage {
            block_types: blocks
        }
    }

    pub fn get_block(&self, pos: BlockInChunkPos) -> BlockID {
        let idx = shared::index(*pos, CHUNK_SIZE);
        self.block_types[idx]
    }
}