use simdnoise::NoiseBuilder;

use crate::entities::{
    name_to_block_id, BlockID, BlockInChunkPos, BlockPos, BlockStorage, ChunkPos, CHUNK_SIZE
};

#[derive(Debug, Default, Clone)]
pub struct TerrainConfig {
    seed: i32,
    freq: f32,
    lacunarity: f32,
    octaves: u8,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 1337,
            freq: 0.5,
            lacunarity: 0.5,
            octaves: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    config: TerrainConfig
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        Self {
            config,
        }
    }

    pub fn generate_terrain(&mut self, chunk_pos: ChunkPos) -> BlockStorage {
        let noise = self.generate_density_map(chunk_pos);

        let mut blocks = BlockStorage::default().get_blocks().to_owned();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let world_x: isize = x as isize + chunk_pos.x;
                    let world_y: isize = y as isize + chunk_pos.y;
                    let world_z: isize = z as isize + chunk_pos.z;

                    let pos = BlockInChunkPos::new(x, y, z);
                    let world_pos = BlockPos::new(world_x, world_y, world_z);
                    let block_id = self.generate_voxel(pos, world_pos, noise[pos.index()]);

                    blocks[pos.index()] = block_id;
                }
            }
        }

        BlockStorage::new(blocks)
    }
    
    fn generate_density_map(&self, chunk_pos: ChunkPos) -> Vec<f32> {
        let offset_x = chunk_pos.x as f32;
        let offset_y = chunk_pos.y as f32;
        let offset_z = chunk_pos.z as f32;

        NoiseBuilder::fbm_3d_offset(offset_x, CHUNK_SIZE, offset_y, CHUNK_SIZE, offset_z, CHUNK_SIZE)
            .with_freq(0.5)
            .with_octaves(5)
            .with_seed(1337)
            .with_lacunarity(0.5)
            .generate_scaled(0.0, 100.0)
    }

    fn generate_voxel(&self, pos: BlockInChunkPos, world_pos: BlockPos, density: f32) -> BlockID {
        if density < 20.0 {
            name_to_block_id("stone")
        } else if density < 40.0 {
            name_to_block_id("grass")
        } else {
            name_to_block_id("air")
        }
    }
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self::new(TerrainConfig::default())
    }
}
