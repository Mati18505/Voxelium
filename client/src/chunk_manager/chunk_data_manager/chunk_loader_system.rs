use bevy::prelude::*;

use crate::{
    bevy_types::AppStates,
};
use shared::chunk_io::{ChunkLoader, ChunkProvider};

use super::events::*;
use super::resources::*;

pub struct ChunkLoaderPlugin;
impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChunkLoaded>()
            .add_event::<ChunkLoaderRequest>()
            .add_systems(Update, chunks_loader.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Resource)]
pub struct ChunkLoaderResource {
    chunk_loader: ChunkLoader,
}

impl ChunkLoaderResource {
    pub fn new(chunk_provider: Box<dyn ChunkProvider>) -> Self {
        let chunk_loader = ChunkLoader::new(chunk_provider);

        Self { chunk_loader }
    }
}

fn chunks_loader(
    mut chunk_loaded_ev: EventWriter<ChunkLoaded>,
    mut chunk_load_req_ev: EventReader<ChunkLoaderRequest>,
    mut chunk_loader: ResMut<ChunkLoaderResource>,
    chunk_manager_resource: Res<ChunkManagerResource>,
) {
    let mut chunk_loader = &mut chunk_loader.chunk_loader;

    for ev in chunk_load_req_ev.read() {
        match ev {
            ChunkLoaderRequest::Load(chunk_pos) => {
                chunk_loader.load_chunk(*chunk_pos);
            }
            ChunkLoaderRequest::CancelLoading(chunk_pos) => {
                chunk_loader.cancel_loading_chunk(*chunk_pos);
            }
        }
    }

    chunk_loader.update(chunk_manager_resource.controller_pos);

    for (chunk_pos, chunk) in chunk_loader.poll_loaded_chunks() {
        chunk_loaded_ev.write(ChunkLoaded { chunk_pos, chunk });
    }
}
