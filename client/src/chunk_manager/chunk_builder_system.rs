use std::sync::Arc;

use bevy::prelude::*;

use crate::bevy_types::AppStates;
use crate::chunk_manager::events::*;
use crate::chunk_manager::resources::ChunkManagerResource;
use crate::chunk_mesh_builder::builders::async_chunk_builder::AsyncChunkBuilder;
use crate::chunk_mesh_builder::builders::{self, ChunkBuilder, Versioned};
use crate::chunk_mesh_builder::meshers::ChunkMesher;

pub struct ChunkBuilderPlugin;
impl Plugin for ChunkBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChunkBuilt>()
            .add_event::<ChunkBuildRequest>()
            .add_systems(Update, chunks_builder.run_if(in_state(AppStates::InGame)));
    }
}

pub trait VersionedChunkBuilder<T: Send + Sync + Default>: ChunkBuilder<T> + Versioned<T> {}

impl<T: Send + Sync + Default, U> VersionedChunkBuilder<T> for U where
    U: ChunkBuilder<T> + Versioned<T>
{
}

#[derive(Resource)]
pub struct ChunkBuilderResource {
    chunk_builder: Box<dyn VersionedChunkBuilder<()>>,
}

impl ChunkBuilderResource {
    pub fn new(voxel_mesher: Arc<dyn ChunkMesher>) -> Self {
        let inner_builder = Box::new(AsyncChunkBuilder::new(voxel_mesher));
        let chunk_builder = Box::new(
            builders::versioned_chunk_builder::VersionedChunkBuilder::<()>::new(inner_builder),
        );

        Self { chunk_builder }
    }
}

fn chunks_builder(
    mut chunk_built_ev: EventWriter<ChunkBuilt>,
    mut chunk_build_req_ev: EventReader<ChunkBuildRequest>,
    mut chunk_builder: ResMut<ChunkBuilderResource>,
    chunk_manager_resource: Res<ChunkManagerResource>,
) {
    let mut chunk_builder = &mut chunk_builder.chunk_builder;

    for ev in chunk_build_req_ev.read() {
        chunk_builder.force_build(ev.chunk_pos, &ev.chunk, ());
    }

    chunk_builder.update(chunk_manager_resource.controller_pos);

    for (chunk_pos, (chunk_mesh, _)) in chunk_builder.poll_completed() {
        chunk_built_ev.write(ChunkBuilt {
            chunk_pos,
            chunk_mesh,
        });
    }
}
