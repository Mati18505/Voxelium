use bevy::prelude::*;

use bevy::diagnostic::{
    Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic, DEFAULT_MAX_HISTORY_LENGTH,
};

use crate::chunk_manager::ChunkMeshes;

#[derive(Resource, Debug, Clone)]
pub struct ChunkMeshDiagnosticsConfig {
    /// The total number of values to keep.
    pub max_history_length: usize,
}

impl Default for ChunkMeshDiagnosticsConfig {
    fn default() -> Self {
        Self {
            max_history_length: DEFAULT_MAX_HISTORY_LENGTH,
        }
    }
}

#[derive(Default)]
pub struct ChunkMeshDiagnosticsPlugin(pub ChunkMeshDiagnosticsConfig);

impl Plugin for ChunkMeshDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .register_diagnostic(
                Diagnostic::new(Self::CHUNK_MESH_COUNT)
                    .with_max_history_length(self.0.max_history_length),
            )
            .register_diagnostic(
                Diagnostic::new(Self::VERTEX_COUNT)
                    .with_max_history_length(self.0.max_history_length),
            )
            .add_systems(Update, Self::diagnostic_system);
    }
}

impl ChunkMeshDiagnosticsPlugin {
    /// Number of currently allocated entities.
    pub const CHUNK_MESH_COUNT: DiagnosticPath = DiagnosticPath::const_new("chunk_mesh_count");
    pub const VERTEX_COUNT: DiagnosticPath = DiagnosticPath::const_new("vertex_count");

    pub fn diagnostic_system(mut diagnostics: Diagnostics, chunk_meshes: Res<ChunkMeshes>) {
        let chunk_mesh_count: usize = chunk_meshes.0.iter().count();

        let vertex_count: usize = chunk_meshes
            .0
            .iter()
            .map(|chunk_mesh| chunk_mesh.1.vertex_count())
            .sum();

        diagnostics.add_measurement(&Self::CHUNK_MESH_COUNT, || chunk_mesh_count as f64);
        diagnostics.add_measurement(&Self::VERTEX_COUNT, || vertex_count as f64);
    }
}
