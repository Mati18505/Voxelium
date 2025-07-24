/// State of ChunkMesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshState {
    /// The `ChunkMesh` is not built and is not queued.
    Empty,

    /// The `ChunkMesh` is waiting to be built (e.g. waiting for neighbors to load).
    WaitingToBuild,

    /// The `ChunkMesh` is in builder queue.
    Building,

    /// The `ChunkMesh` is built and stored in world.
    Built,
}

/// External inputs for determining next `MeshState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkMeshStatus {
    /// Whether the chunk is inside the current render distance.
    pub is_within_render: bool,

    /// Whether all preconditions are met to begin building the mesh.
    pub can_start_building: bool,

    /// Whether the mesh building task has completed.
    pub mesh_built: bool,

    /// Whether the current mesh needs to be rebuilt (e.g. voxel edit / neighbor changed).
    pub needs_rebuild: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshTransition {
    pub from: MeshState,
    pub to: MeshState,
}

impl MeshState {
    pub fn get_next_chunk_mesh_state(&self, status: ChunkMeshStatus) -> MeshState {
        use MeshState::*;

        let curr_state = self;

        if !status.is_within_render {
            return Empty;
        }

        match curr_state {
            Empty => {
                WaitingToBuild
            }
            WaitingToBuild => {
                if status.can_start_building {
                    Building
                } else {
                    curr_state
                }
            }
            Building => {
                if status.needs_rebuild {
                    return WaitingToBuild;
                }

                if status.mesh_built {
                    Built
                } else {
                    curr_state
                }
            }
            Built => {
                if status.needs_rebuild {
                    WaitingToBuild
                } else {
                    curr_state
                }
            }
        }
    }

    pub fn get_mesh_transition(&self, to: MeshState) -> Option<MeshTransition> {
        use MeshState::*;

        let from = self;

        match (from, to) {
            (WaitingToBuild, Empty)
            | (WaitingToBuild, Building)
            | (Building, Empty)
            | (Building, WaitingToBuild)
            | (Building, Built)
            | (Built, Empty)
            | (Built, WaitingToBuild) => Some(MeshTransition { from, to }),
            _ => None,
        }
    }
}