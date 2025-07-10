use std::sync::{Arc, Mutex};
use shared::chunk_loader::ChunkLoader;
use shared::entities::{Chunk, ChunkPos, ChunkRepository};

use super::physical_world::PhysicalWorld;
use super::{ChunkBuilder, ChunkObjectCallback, Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Loaded,
    ToDraw,
    Drawn,
}

pub struct ChunkManagerContext<'a> {
    pub chunk_loader: &'a mut ChunkLoader,
    pub chunk_builder: &'a mut dyn ChunkBuilder,
    pub world: &'a mut PhysicalWorld,
    pub chunk_object_callback: &'a Option<Arc<Mutex<dyn ChunkObjectCallback>>>,
    pub config: &'a Config,
}
impl<'a> ChunkManagerContext<'a> {
    fn with_chunk_object_callback<F: FnOnce(&mut dyn ChunkObjectCallback)>(&self, f: F)
    {
        if let Some(cb) = &self.chunk_object_callback {
            if let Ok(mut cb) = cb.lock() {
                f(&mut *cb);
            }
        }
    }
    pub fn load_chunk(&mut self, pos: ChunkPos) -> Chunk {
        self.chunk_loader.load_chunk(pos)
    }
}

static LOADED: Loaded = Loaded {};
static TO_DRAW: ToDraw = ToDraw {};
static DRAWN: Drawn = Drawn {};

pub fn chunk_state_to_behavior(state: ChunkState) -> &'static dyn ChunkStateBehavior {
    match state {
        ChunkState::Loaded => &LOADED,
        ChunkState::ToDraw => &TO_DRAW,
        ChunkState::Drawn => &DRAWN,
    }
}

pub struct Loaded {}

pub struct ToDraw {}

pub struct Drawn {}

pub trait ChunkStateBehavior: Send + Sync {
    fn on_enter(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos);
    fn on_exit(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos);
    fn update(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos, controller_pos: ChunkPos) -> ChunkState;
    // Redraws chunk if needed.
    fn notify_chunk_modified(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos);
}

impl ChunkStateBehavior for Loaded {
    fn on_enter(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
        let chunk = manager_context.load_chunk(pos);

        manager_context.world.set_chunk(pos, chunk);
    }

    fn on_exit(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
    }
    
    fn update(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos, controller_pos: ChunkPos) -> ChunkState {
        let new_mesh_version = manager_context.world.increment_chunk_mesh_version(pos);

        if let Some(chunk_to_build) = manager_context.world.get_chunk(pos) {
            manager_context.chunk_builder.build_chunk(pos, chunk_to_build, new_mesh_version);
            return ChunkState::ToDraw;
        } else {
            eprintln!("Chunk to build don't exist in world!");
        }

        ChunkState::Loaded
    }

    fn notify_chunk_modified(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
        // No action needed for Loaded state
    }
}

impl ChunkStateBehavior for ToDraw {
    fn on_enter(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
    }

    fn on_exit(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
    }    

    fn update(&self, manager_context: &mut ChunkManagerContext, chunk_pos: ChunkPos, controller_pos: ChunkPos) -> ChunkState {
        if let Some((chunk_mesh, mesh_version)) = manager_context.chunk_builder.take_builded_chunk_mesh(chunk_pos) {
            if chunk_pos.is_within_distance(controller_pos, manager_context.config.render_distance) {
                if mesh_version == manager_context.world.get_chunk_mesh_version(chunk_pos) {
                    manager_context.world.add_chunk_mesh(chunk_pos, chunk_mesh.clone());
                    manager_context.with_chunk_object_callback(|cb| cb.chunk_object_created(chunk_pos, &chunk_mesh));

                    return ChunkState::Drawn;
                }
            } else {
                return ChunkState::Loaded;
            }
        }

        ChunkState::ToDraw
    }

    fn notify_chunk_modified(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
        // self.world.change_chunk_state(pos, super::ChunkState::ToDraw);

        //     let new_mesh_version: Version = self.world.increment_chunk_mesh_version(pos);
        //     self.chunk_builder.build_chunk(pos, &new_chunk, new_mesh_version);
    }
}

impl ChunkStateBehavior for Drawn {
    fn on_enter(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
        // add chunk mesh from constructor
    }

    fn on_exit(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
        manager_context.world.chunk_meshes.remove(&pos);
        manager_context.with_chunk_object_callback(|cb| cb.chunk_object_removed(pos));
    }

    fn update(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos, controller_pos: ChunkPos) -> ChunkState {
        match pos.is_within_distance(controller_pos, manager_context.config.render_distance) {
            true => ChunkState::Drawn,
            false => ChunkState::Loaded,
        }
    }

    fn notify_chunk_modified(&self, manager_context: &mut ChunkManagerContext, pos: ChunkPos) {
        // self.world.change_chunk_state(pos, super::ChunkState::ToDraw);

        //     let new_mesh_version: Version = self.world.increment_chunk_mesh_version(pos);
        //     self.chunk_builder.build_chunk(pos, &new_chunk, new_mesh_version);
    }
}