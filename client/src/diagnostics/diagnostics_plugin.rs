use bevy::{
    app::{App, Plugin},
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    ecs::resource::Resource,
};

use crate::diagnostics::ChunkMeshDiagnosticsPlugin;

#[derive(Resource, Debug, Clone)]
pub struct DiagnosticsConfig {}

pub struct DiagnosticsPlugin(DiagnosticsConfig);
impl DiagnosticsPlugin {
    pub fn new(config: DiagnosticsConfig) -> Self {
        Self(config)
    }
}
impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone()).add_plugins((
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin,
            ChunkMeshDiagnosticsPlugin::default(),
        ));
    }
}
