use super::bevy_controller_events::*;
use bevy::prelude::*;
use bevy_flycam::*;

pub struct ControllerPlugin;
impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NoCameraPlayerPlugin)
            .insert_resource(MovementSettings {
                speed: 100.0,
                ..default()
            })
            .add_systems(Startup, setup_controller)
            .add_systems(Update, (update, player_action));
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Controller {
    last_player_pos: Vec3,
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
    mut commands: Commands,
    q_fly_cam: Query<&Transform, With<FlyCam>>,
) {
    if let Ok(mut controller) = q_controller.single_mut() {
        if let Ok(transform) = q_fly_cam.single() {
            if controller.last_player_pos.floor() != transform.translation.floor() {
                commands.trigger(PositionChangeEvent {
                    prev_pos: controller.last_player_pos,
                    new_pos: transform.translation,
                });

                controller.last_player_pos = transform.translation;
            }
        }
    } else {
        warn!("Controller not found for 'update'!");
    }
}

pub fn player_action(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    q_fly_cam: Query<&Transform, With<FlyCam>>,
) {
    if let Ok(transform) = q_fly_cam.single() {
        if mouse.just_pressed(MouseButton::Left) {
            commands.trigger(ActionEvent {
                action_type: ActionType::LeftClick,
                controller_forward: *transform.forward(),
                controller_pos: transform.translation,
            });
        } else if mouse.just_pressed(MouseButton::Right) {
            commands.trigger(ActionEvent {
                action_type: ActionType::RightClick,
                controller_forward: *transform.forward(),
                controller_pos: transform.translation,
            });
        }
    }
}
