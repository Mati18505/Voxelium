use rand::{rngs::ThreadRng, Rng};

use crate::entities::{
    block_in_chunk_pos_generator::BlockInChunkPosGenerator, name_to_block_id, BlockID,
    BlockStorage, ChunkPos, CHUNK_SIZE,
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

    #[allow(unused)]
    pub fn generate_terrain(&mut self, chunk_pos: ChunkPos) -> BlockStorage {
        let mut blocks = BlockStorage::default().get_blocks().to_owned();
        let height_map = self.generate_height_map();

        for pos in BlockInChunkPosGenerator::new() {
            let world_x: isize = pos.x as isize + chunk_pos.x;
            let world_y: isize = pos.y as isize + chunk_pos.y;
            let world_z: isize = pos.z as isize + chunk_pos.z;

            let height_map_index = Self::index_height_map(pos.x, pos.z);
            let generated_height: i64 = height_map[height_map_index];
            let block_id = self.generate_voxel(world_y as i64, generated_height);

            blocks[pos.index()] = block_id;
        }

        BlockStorage::new(blocks)
    }

    #[allow(unused)]
    fn generate_height_map(&mut self) -> Vec<i64> {
        let mut result = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let height = self.engine.random_range(5..16);
                result.push(height);
            }
        }

        result
    }

    fn index_height_map(x: usize, z: usize) -> usize {
        z * CHUNK_SIZE + x
    }

    fn generate_voxel(&mut self, world_y: i64, generated_height: i64) -> BlockID {
        /*
            match world_z {
                world_z if world_z > generated_height => biome.atmosphereBlock,
                world_z if world_z == generated_height => biome.layer1stBlock,
                world_z if world_z >= generated_height - 5 => biome.layer2ndBlock,
                world_z if world_z < generated_height => biome.layer3rdBlock,
                _ => 0
            }
        */

        if world_y.abs() < generated_height - 4 {
            name_to_block_id("red")
        } else if world_y.abs() < generated_height - 2 {
            name_to_block_id("wood")
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
