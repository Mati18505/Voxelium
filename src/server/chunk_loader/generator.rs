use rand::Rng;
use voxelium::entities::{BlockID, BlockInChunkPos, BlockStorage, ChunkPos, CHUNK_SIZE};

#[derive(Debug, Clone, PartialEq)]
pub struct Generator {
    chunk_pos: ChunkPos,
    block_storage: BlockStorage,
}

impl Generator {
    pub fn new(chunk_pos: ChunkPos) -> Self {
        Generator {
            chunk_pos,
            block_storage: BlockStorage::default(),
        }
    }

    pub fn generate_terrain(mut self) -> Self {
        let mut blocks = self.block_storage.get_blocks().to_owned();

        for y in 0..CHUNK_SIZE
        {
            for x in 0..CHUNK_SIZE
            {
                let world_x: isize = x as isize + self.chunk_pos.x;
                let world_y: isize = y as isize + self.chunk_pos.y;

                let generated_height = Self::generate_height(world_x, world_y);

                for z in 0..CHUNK_SIZE
                {
                    let world_z: isize = z as isize + self.chunk_pos.z;
                    let block_id = Self::generate_voxel(world_z, generated_height);
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

    fn generate_height(_world_x: isize, _world_y: isize) -> isize {
        rand::thread_rng().gen_range(5..16);
        0
    }

    fn generate_voxel(world_z: isize, generated_height: isize) -> BlockID
    {
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
            rand::thread_rng().gen_range(1..=8)
        } else {
            0
        }
    }
}