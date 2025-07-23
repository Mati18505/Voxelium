use bevy::prelude::*;
use shared::chunk_io::{
    providers::{
        generated_chunk_provider::GeneratedChunkProvider, terrain_generator::TerrainGenerator,
    },
    ChunkLoader,
};

use crate::{
    bevy_types::{AppStates, GameResources},
    chunk_manager::ChunkManagerResources,
    controller::Controller,
    gui::UIGeneratorDataEvent,
};

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reload_generator.run_if(in_state(AppStates::InGame)));
    }
}

fn reload_generator(
    mut ui_change_ev: EventReader<UIGeneratorDataEvent>,
    mut chunk_manager_resources: ResMut<ChunkManagerResources>,
) {
    for ev in ui_change_ev.read() {
        info!("generator update {:?}", &ev.terrain);

        let new_generator = TerrainGenerator::new(ev.terrain.clone());
        let new_provider = Box::new(GeneratedChunkProvider::new(new_generator));
        chunk_manager_resources
            .chunk_manager
            .change_chunk_loader_and_reload_all(ChunkLoader::new(new_provider));
    }
}
