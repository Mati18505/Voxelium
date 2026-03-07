use bevy::prelude::*;
use shared::entities::name_to_block_id;

use super::{super::utils::raycast_from_controller, super::LookedAtBlockChangedEvent};
use crate::{
    bevy_types::GameResources,
    chunk_manager::{ChunkManagerResources, WorldChunkUpdateEvent},
    controller::Controller,
};

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LookedAtBlockData {
    last_player_pos: Vec3,
    last_looking_dir: Vec3,
}

pub fn initialize_looked_at_block(mut commands: Commands) {
    commands.spawn(LookedAtBlockData::default());
}

pub fn update_looked_at_block(
    mut commands: Commands,
    mut q_looked_at_block_data: Query<&mut LookedAtBlockData>,
    q_controller: Query<&Transform, With<Controller>>,
    world_chunk_update_ev: MessageReader<WorldChunkUpdateEvent>,
    chunk_manager_resources: Res<ChunkManagerResources>,
    game_resources: Res<GameResources>,
) {
    let transform = match q_controller.single() {
        Ok(t) => t,
        Err(_) => {
            warn!("Controller with Transform not found for 'update_looked_at_block'!");
            return;
        }
    };

    let mut looked_at_block_data = match q_looked_at_block_data.single_mut() {
        Ok(data) => data,
        Err(_) => {
            warn!("LookedAtBlockData not found for 'update_looked_at_block'!");
            return;
        }
    };

    let pos = transform.translation;
    let dir = Vec3::from(transform.forward());

    let moved = looked_at_block_data.last_player_pos != pos;
    let looking_dir_changed = looked_at_block_data.last_looking_dir != dir;
    let world_updated = !world_chunk_update_ev.is_empty();

    looked_at_block_data.last_looking_dir = dir;
    looked_at_block_data.last_player_pos = pos;

    let dirty = moved || looking_dir_changed || world_updated;

    if dirty {
        process_raycast_and_send_event(
            commands,
            chunk_manager_resources,
            game_resources,
            looked_at_block_data,
        );
    }
}

fn process_raycast_and_send_event(
    mut commands: Commands,
    chunk_manager_resources: Res<'_, ChunkManagerResources>,
    game_resources: Res<'_, GameResources>,
    looked_at_block_data: Mut<'_, LookedAtBlockData>,
) {
    let world = &chunk_manager_resources.chunk_manager.get_world().world;
    let raycast_result = raycast_from_controller(
        looked_at_block_data.last_player_pos,
        looked_at_block_data.last_looking_dir,
        world,
        &game_resources.server_block_type_storage,
    );
    let block_pos = raycast_result.hitpoint.pos;
    let block_id = raycast_result.hitpoint.block_id;

    if let Some(default_block_type) = game_resources
        .server_block_type_storage
        .get_by_id(name_to_block_id("air"))
    {
        if raycast_result.collide {
            let block_type = match game_resources.server_block_type_storage.get_by_id(block_id) {
                Some(block_type) => block_type,
                None => {
                    warn!("Block type ID {block_id} not found in block type storage!");
                    default_block_type
                }
            };

            commands.trigger(LookedAtBlockChangedEvent {
                block_pos,
                block_type: block_type.clone(),
            });
        } else {
            commands.trigger(LookedAtBlockChangedEvent {
                block_pos,
                block_type: default_block_type.clone(),
            });
        }
    } else {
        warn!("Air block type not found in block type storage!")
    }
}
