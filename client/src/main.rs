use bevy::{
    app::{App, Startup},
    prelude::*,
    DefaultPlugins,
};
use controller::ControllerPlugin;

mod chunk_builder;
mod controller;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            ControllerPlugin,
        ))
        .add_systems(Startup, init_level)
        .run();
}


fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(2.5)))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        GlobalTransform::default(),
    ));

    commands.spawn((
        PointLight { ..default() },
        Transform::from_xyz(10.0, 20.0, 4.0),
        GlobalTransform::default(),
    ));
}