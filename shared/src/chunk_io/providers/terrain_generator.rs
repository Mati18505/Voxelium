use simdnoise::NoiseBuilder;

use crate::entities::{
    block_in_chunk_pos_generator::BlockInChunkPosGenerator, BlockID, BlockInChunkPos, BlockPos,
    BlockRegistry, BlockStorage, ChunkPos, CHUNK_SIZE,
};

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn generate_terrain(
        &mut self,
        chunk_pos: ChunkPos,
        registry: &dyn BlockRegistry,
    ) -> BlockStorage {
        let density_noise = self.generate_density_map(chunk_pos);
        let flat_noise = self.generate_flat_map(chunk_pos);

        let mut blocks = vec![0; CHUNK_SIZE.pow(3)];

        for block_in_chunk_pos in BlockInChunkPosGenerator::new() {
            let world_pos = Self::get_world_pos(chunk_pos, block_in_chunk_pos);
            let block_id = self.generate_voxel_flat(world_pos, flat_noise[Self::index_2d(block_in_chunk_pos)], registry);

            blocks[block_in_chunk_pos.index()] = block_id;
        }

        BlockStorage::new(blocks)
    }

    fn get_world_pos(chunk_pos: ChunkPos, block_in_chunk_pos: BlockInChunkPos) -> BlockPos {
        let world_x: isize = block_in_chunk_pos.x as isize + chunk_pos.x;
        let world_y: isize = block_in_chunk_pos.y as isize + chunk_pos.y;
        let world_z: isize = block_in_chunk_pos.z as isize + chunk_pos.z;

        BlockPos::new(world_x, world_y, world_z)
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

    fn generate_voxel_flat(
        &self,
        world_pos: BlockPos,
        flat: f32,
        registry: &dyn BlockRegistry,
    ) -> BlockID {
        if world_pos.y < flat.round() as isize {
            registry.name_to_block_id("stone")
        } else {
            registry.name_to_block_id("air")
        }
    }

    fn generate_voxel_with_flat(
        &self,
        pos: BlockInChunkPos,
        world_pos: BlockPos,
        density: f32,
        flat: f32,
        registry: &dyn BlockRegistry,
    ) -> BlockID {
        if world_pos.y < flat.round() as isize {
            if density < 20.0 {
                registry.name_to_block_id("stone")
            } else if density < 40.0 {
                registry.name_to_block_id("grass")
            } else {
                registry.name_to_block_id("air")
            }
        } else {
            registry.name_to_block_id("air")
        }
    }

    fn generate_voxel(
        &self,
        pos: BlockInChunkPos,
        world_pos: BlockPos,
        density: f32,
        registry: &dyn BlockRegistry,
    ) -> BlockID {
        if density < 20.0 {
            registry.name_to_block_id("stone")
        } else if density < 40.0 {
            registry.name_to_block_id("grass")
        } else {
            registry.name_to_block_id("air")
        }
    }
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self::new(TerrainConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_world_pos() {
        let chunk_pos = ChunkPos::new(16, 32, 48);
        let block_pos = BlockInChunkPos::new(1, 2, 3);

        let result = TerrainGenerator::get_world_pos(chunk_pos, block_pos);

        assert_eq!(result, BlockPos::new(17, 34, 51));
    }

    #[test]
    fn test_get_world_pos_max() {
        let chunk_pos = ChunkPos::new(16, 16, 16);
        let block_pos = BlockInChunkPos::new(15, 15, 15);

        let result = TerrainGenerator::get_world_pos(chunk_pos, block_pos);

        assert_eq!(result, BlockPos::new(31, 31, 31));
    }

    #[test]
    fn test_get_world_pos_negative_chunk() {
        let chunk_pos = ChunkPos::new(-16, -32, -48);
        let block_pos = BlockInChunkPos::new(0, 0, 0);

        let result = TerrainGenerator::get_world_pos(chunk_pos, block_pos);

        assert_eq!(result, BlockPos::new(-16, -32, -48));
    }

    #[test]
    fn test_get_world_pos_negative_chunk_with_offset() {
        let chunk_pos = ChunkPos::new(-16, -16, -16);
        let block_pos = BlockInChunkPos::new(15, 15, 15);

        let result = TerrainGenerator::get_world_pos(chunk_pos, block_pos);

        assert_eq!(result, BlockPos::new(-1, -1, -1));
    }

    #[test]
    fn test_get_world_pos_mixed_axes() {
        let chunk_pos = ChunkPos::new(32, -16, 0);
        let block_pos = BlockInChunkPos::new(4, 8, 12);

        let result = TerrainGenerator::get_world_pos(chunk_pos, block_pos);

        assert_eq!(result, BlockPos::new(36, -8, 12));
    }
}
