use cgmath::num_traits::Float;
use simdnoise::{CellDistanceFunction, NoiseBuilder, FbmSettings};

use crate::entities::{
    name_to_block_id, BlockID, BlockInChunkPos, BlockPos, BlockStorage, ChunkPos, CHUNK_SIZE
};

#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    chunk_pos: ChunkPos,
    height_map: Vec<f32>,
    density_map: Vec<f32>,
}

impl TerrainGenerator {
    pub fn new() -> Self {
        Self {
            chunk_pos: ChunkPos::new(0, 0, 0),
            height_map: Vec::default(),
            density_map: Vec::default(),
        }
    }

    pub fn generate_terrain(&mut self, chunk_pos: ChunkPos) -> BlockStorage {
        self.chunk_pos = chunk_pos;
        self.generate_noise();

        let mut blocks = BlockStorage::default().get_blocks().to_owned();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let world_x: isize = x as isize + chunk_pos.x;
                    let world_y: isize = y as isize + chunk_pos.y;
                    let world_z: isize = z as isize + chunk_pos.z;

                    let pos = BlockInChunkPos::new(x, y, z);
                    let world_pos = BlockPos::new(world_x, world_y, world_z);
                    let block_id = self.generate_voxel(pos, world_pos);

                    blocks[pos.index()] = block_id;
                }
            }
        }

        BlockStorage::new(blocks)
    }
    
    fn generate_noise(&mut self) {
        let offset_x = self.chunk_pos.x as f32;
        let offset_y = self.chunk_pos.y as f32;
        let offset_z = self.chunk_pos.z as f32;

        self.density_map = NoiseBuilder::fbm_3d_offset(offset_x, CHUNK_SIZE, offset_y, CHUNK_SIZE, offset_z, CHUNK_SIZE)
            .with_freq(0.5)
            .with_octaves(5)
            .with_seed(1337)
            .with_lacunarity(0.5)
            .generate_scaled(0.0, 100.0);
    }

    fn generate_voxel(&self, pos: BlockInChunkPos, world_pos: BlockPos) -> BlockID {
        if self.density_map[pos.index()] < 20.0 {
            name_to_block_id("stone")
        } else if self.density_map[pos.index()] < 40.0 {
            name_to_block_id("grass")
        } else {
            name_to_block_id("air")
        }
    }
    fn index(pos: BlockInChunkPos) -> usize {
        pos.x + pos.z * CHUNK_SIZE + pos.y * CHUNK_SIZE * CHUNK_SIZE
    }
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self::new()
    }
}
