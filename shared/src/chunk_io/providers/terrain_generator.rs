use bevy::log::{info_span, tracing::span};
use simdnoise::NoiseBuilder;

use crate::entities::{
    name_to_block_id, BlockID, BlockInChunkPos, BlockPos, BlockStorage, ChunkPos, CHUNK_SIZE,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainConfig {
    pub seed: i32,
    pub freq: f32,
    pub lacunarity: f32,
    pub octaves: u8,
    pub add_flat_noise: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 1337,
            freq: 0.630,
            lacunarity: 0.5,
            octaves: 5,
            add_flat_noise: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    config: TerrainConfig,
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        Self { config }
    }

    pub fn generate_terrain(&mut self, chunk_pos: ChunkPos) -> BlockStorage {
        let my_span = info_span!("generate_terrain", name = "generate_terrain").entered();
        let density_noise = self.generate_density_map(chunk_pos);
        let flat_noise = self.generate_flat_map(chunk_pos);

        let mut blocks = vec![0; CHUNK_SIZE.pow(3)];

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let world_x: isize = x as isize + chunk_pos.x;
                    let world_y: isize = y as isize + chunk_pos.y;
                    let world_z: isize = z as isize + chunk_pos.z;

                    let pos = BlockInChunkPos::new(x, y, z);
                    let world_pos = BlockPos::new(world_x, world_y, world_z);
                    let block_id = match self.config.add_flat_noise {
                        true => self.generate_voxel_with_flat(
                            pos,
                            world_pos,
                            density_noise[pos.index()],
                            flat_noise[Self::index_2d(pos)],
                        ),
                        false => self.generate_voxel(pos, world_pos, density_noise[pos.index()]),
                    };

                    blocks[pos.index()] = block_id;
                }
            }
        }

        BlockStorage::new(blocks)
    }

    fn index_2d(pos: BlockInChunkPos) -> usize {
        pos.z * CHUNK_SIZE + pos.x
    }

    fn generate_density_map(&self, chunk_pos: ChunkPos) -> Vec<f32> {
        let offset_x = chunk_pos.x as f32;
        let offset_y = chunk_pos.y as f32;
        let offset_z = chunk_pos.z as f32;

        NoiseBuilder::fbm_3d_offset(
            offset_x, CHUNK_SIZE, offset_y, CHUNK_SIZE, offset_z, CHUNK_SIZE,
        )
        .with_freq(self.config.freq)
        .with_octaves(self.config.octaves)
        .with_seed(self.config.seed)
        .with_lacunarity(self.config.lacunarity)
        .generate_scaled(0.0, 100.0)
    }

    fn generate_flat_map(&self, chunk_pos: ChunkPos) -> Vec<f32> {
        let offset_x = chunk_pos.x as f32;
        let offset_z = chunk_pos.z as f32;

        NoiseBuilder::fbm_2d_offset(offset_x, CHUNK_SIZE, offset_z, CHUNK_SIZE)
            .with_freq(self.config.freq)
            .with_octaves(self.config.octaves)
            .with_seed(self.config.seed)
            .with_lacunarity(self.config.lacunarity)
            .generate_scaled(0.0, 100.0)
    }

    fn generate_voxel_with_flat(
        &self,
        pos: BlockInChunkPos,
        world_pos: BlockPos,
        density: f32,
        flat: f32,
    ) -> BlockID {
        if world_pos.y < flat.round() as isize {
            if density < 20.0 {
                name_to_block_id("stone")
            } else if density < 40.0 {
                name_to_block_id("grass")
            } else {
                name_to_block_id("air")
            }
        } else {
            name_to_block_id("air")
        }
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
