use crate::entities::{BlockID, BlockInChunkPos, BlockStorage, ChunkPos, CHUNK_SIZE};

use rand::Rng;
use cgmath::Vector3;

pub fn generate(chunk_pos: &ChunkPos) -> BlockStorage {
    let mut chunk_blocks = vec![0; CHUNK_SIZE.pow(3)];

    for y in 0..CHUNK_SIZE
    {
        for x in 0..CHUNK_SIZE
        {
            let world_x: isize = x as isize + chunk_pos.x;
            let world_y: isize = y as isize + chunk_pos.y;

            let generated_height = generate_height(world_x, world_y);

            for z in 0..CHUNK_SIZE
            {
                let world_z: isize = z as isize + chunk_pos.z;
                let block_id = generate_voxel(world_z, generated_height);
                let pos = BlockInChunkPos(Vector3::new(x, y, z));

                chunk_blocks[pos.index()] = block_id;
            }

        }
    }
    BlockStorage::new(chunk_blocks)
}

fn generate_height(world_x: isize, world_y: isize) -> isize {
    let mut rng = rand::thread_rng();
    return rng.gen_range(5..16);
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
        1
    } else {
        0
    }
}