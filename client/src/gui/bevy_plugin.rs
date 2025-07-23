use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

use crate::{bevy_types::AppStates, orchestrator};
use shared::{chunk_io::providers::terrain_generator::TerrainConfig, entities::*};

pub struct GUIPlugin;
impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_event::<UIGeneratorDataEvent>()
            .insert_resource(UIGeneratorData::default())
            .add_systems(Startup, setup_gui)
            .add_systems(
                Update,
                update_block_cursor.run_if(in_state(AppStates::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                terrain_editor.run_if(in_state(AppStates::InGame)),
            );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BlockCursorData {
    visible: bool,
    block_pos: BlockPos,
}

impl Default for BlockCursorData {
    fn default() -> Self {
        Self {
            visible: false,
            block_pos: BlockPos::new(0, 0, 0),
        }
    }
}

fn setup_gui(mut commands: Commands) {
    commands.spawn((BlockCursorData::default(),));
}

fn update_block_cursor(
    mut gizmos: Gizmos,
    mut looked_at_block_change_ev: EventReader<orchestrator::LookedAtBlockChangedEvent>,
    mut q_block_cursor_data: Query<&mut BlockCursorData>,
) {
    let mut block_cursor_data = match q_block_cursor_data.single_mut() {
        Ok(block_cursor_data) => block_cursor_data,
        Err(_) => {
            warn!("Gizmo data not found for update_gizmo!");
            return;
        }
    };

    for ev in looked_at_block_change_ev.read() {
        block_cursor_data.visible = ev.block_type.affect_raycast;
        block_cursor_data.block_pos = ev.block_pos;
    }

    if block_cursor_data.visible {
        gizmos.cuboid(
            {
                let translation = Vec3::new(
                    block_cursor_data.block_pos.x as f32,
                    block_cursor_data.block_pos.y as f32,
                    block_cursor_data.block_pos.z as f32,
                );
                Transform {
                    translation,
                    ..Transform::IDENTITY
                }
            },
            Color::WHITE,
        );
    }
}

#[derive(Resource, Debug, Default)]
struct UIGeneratorData {
    pub terrain: TerrainConfig,
}

#[derive(Event, Debug)]
pub struct UIGeneratorDataEvent {
    pub terrain: TerrainConfig,
}

fn terrain_editor(
    mut ui_data: ResMut<UIGeneratorData>,
    mut change_ev: EventWriter<UIGeneratorDataEvent>,
    mut egui_ctx: EguiContexts,
) -> Result {
    let ctx = egui_ctx.ctx_mut()?;
    let mut new = ui_data.terrain.clone();

    egui::SidePanel::left("")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Terrain config");
            ui.add(
                egui::Slider::new(&mut new.seed, 1000..=5000)
                    .text("seed")
                    .drag_value_speed(1.0),
            );
            ui.add(
                egui::Slider::new(&mut new.freq, 0.001..=1.0)
                    .text("freq")
                    .drag_value_speed(0.001),
            );
            ui.add(
                egui::Slider::new(&mut new.lacunarity, 0.1..=1.0)
                    .text("lacunarity")
                    .drag_value_speed(0.01),
            );
            ui.add(egui::DragValue::new(&mut new.octaves).speed(1));
        });

    if new != ui_data.terrain {
        change_ev.write(UIGeneratorDataEvent {
            terrain: new.clone(),
        });

        ui_data.terrain = new;
    }

    Ok(())
}
