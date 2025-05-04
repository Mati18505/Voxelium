use std::ops::Range;

use crate::entities::{BlockID, BlockInChunkPos, BlockStorage, ChunkPos, CHUNK_SIZE};

pub struct Generator {
    chunk_pos: ChunkPos,
    block_storage: BlockStorage,
    noise: Box<dyn Noise<i64>>,
}

impl Generator {
    pub fn new(chunk_pos: ChunkPos, noise: Box<dyn Noise<i64>>) -> Self {
        Generator {
            chunk_pos,
            block_storage: BlockStorage::default(),
            noise,
        }
    }

    pub fn generate_terrain(mut self) -> Self {
        let mut blocks = self.block_storage.get_blocks().to_owned();

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x: isize = x as isize + self.chunk_pos.x;
                let world_y: isize = y as isize + self.chunk_pos.y;

                let generated_height: i64 = self.generate_height(world_x, world_y);

                for z in 0..CHUNK_SIZE {
                    let world_z: isize = z as isize + self.chunk_pos.z;
                    let block_id = self.generate_voxel(world_z as i64, generated_height);
                    let pos = BlockInChunkPos::new(x, y, z);

                    blocks[pos.index()] = block_id;
                }
            }
        }

        self.block_storage = BlockStorage::new(blocks);
        self
    }

    pub fn get(self) -> BlockStorage {
        self.block_storage
    }

    fn generate_height(&mut self, _world_x: isize, _world_y: isize) -> i64 {
        self.noise.gen_range(5..16)
    }

    fn generate_voxel(&mut self, world_z: i64, generated_height: i64) -> BlockID {
        /*
            match world_z {
                world_z if world_z > generated_height => biome.atmosphereBlock,
                world_z if world_z == generated_height => biome.layer1stBlock,
                world_z if world_z >= generated_height - 5 => biome.layer2ndBlock,
                world_z if world_z < generated_height => biome.layer3rdBlock,
                _ => 0
            }
        */
        if world_z < generated_height {
            self.noise.gen_range(1..9) as BlockID
        } else {
            0
        }
    }
}

pub trait Noise<T: Copy> {
    fn gen_range(&mut self, range: Range<T>) -> T;
}
