use std::collections::{HashMap};

use bevy::prelude::*;
use shared::entities::ChunkPos;

use crate::{bevy_types::AppStates, chunk_mesh_builder::ChunkMesh};

#[derive(Message, Debug, Clone, PartialEq)]
pub struct BuildChunk (
    pub ChunkPos
);

#[derive(Message, Debug, Clone, PartialEq)]
pub struct RemoveChunk (
    pub ChunkPos
);

pub struct ChunkBuilderPlugin;
impl Plugin for ChunkBuilderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<BuildChunk>()
            .add_message::<RemoveChunk>()
            .add_systems(Update, process_chunks_to_build.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Resource)]
pub struct BuilderResources {
    pub chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
}

pub fn process_chunks_to_build(mut reader: MessageReader<BuildChunk>) {
    for message in reader.read() {
        let chunk_pos = message.0;

        /*
        let chunk = world
            .get_chunk(pos)
            .expect("Chunk is passed to builder, but it is not loaded.");
        */

        info!("Building chunk {:?}", &chunk_pos);
    }
}

pub fn process_chunks_to_remove(mut reader: MessageReader<RemoveChunk>) {
    for message in reader.read() {
        let chunk_pos = message.0;

        info!("Removing chunk {:?}", &chunk_pos);
    }
}

