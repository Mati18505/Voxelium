use bevy::prelude::*;
use bevy_flycam::*;

pub struct ControllerPlugin;
impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup_controller)
        .add_systems(FixedUpdate, update);
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Controller {
    last_player_pos: Vec3
}

fn setup_controller(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        FlyCam,
        Transform::from_xyz(30.0, 25.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Controller::default(),
    ));
}

pub fn update(
    mut q_controller: Query<&mut Controller>,
    q_fly_cam: Query<&Transform, With<FlyCam>>,
) {
    if let Ok(mut controller) = q_controller.single_mut() {
        if let Ok(transform) = q_fly_cam.single() {
            if controller.last_player_pos.floor() != transform.translation.floor() {
                position_changed(controller.last_player_pos, transform.translation);
                
                controller.last_player_pos = transform.translation;
            }
        }
    } else {
        warn!("Controller not found for 'update'!");
    }
}

fn position_changed(prev_pos: Vec3, new_pos: Vec3) {
    println!("Controller position changed: prev = {prev_pos}, new = {new_pos}");
}