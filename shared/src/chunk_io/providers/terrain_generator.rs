use rand::{rngs::ThreadRng, Rng};

use crate::entities::{
    name_to_block_id, BlockID, BlockInChunkPos, BlockStorage, ChunkPos, CHUNK_SIZE,
};

#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    engine: ThreadRng,
}

impl TerrainGenerator {
    pub fn new() -> Self {
        Self {
            engine: rand::rng(),
        }
    }

    pub fn generate_terrain(&mut self, chunk_pos: ChunkPos) -> BlockStorage {
        let mut blocks = BlockStorage::default().get_blocks().to_owned();

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x: isize = x as isize + chunk_pos.x;
                let world_y: isize = y as isize + chunk_pos.y;

                let generated_height: i64 = self.generate_height(world_x, world_y);

                for z in 0..CHUNK_SIZE {
                    let world_z: isize = z as isize + chunk_pos.z;
                    let block_id = self.generate_voxel(world_z as i64, generated_height);
                    let pos = BlockInChunkPos::new(x, y, z);

                    blocks[pos.index()] = block_id;
                }
            }
        }

        BlockStorage::new(blocks)
    }

    fn generate_height(&mut self, _world_x: isize, _world_y: isize) -> i64 {
        self.engine.random_range(5..16)
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
        if world_z.abs() < generated_height {
            //self.noise.gen_range(4..5) as BlockID
            name_to_block_id("grass")
        } else {
            name_to_block_id("air")
        }
    }
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self::new()
    }
}
