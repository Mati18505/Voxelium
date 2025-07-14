#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Empty,
    Loading,
    Loaded,
    ToDraw,
    Drawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkStatus {
    pub is_within_render: bool,
    pub is_within_load: bool,
    pub loaded: bool,
    pub mesh_built: bool,
    pub needs_rebuild: bool,
}

pub fn get_next_chunk_state(
    curr_state: ChunkState,
    status: ChunkStatus,
) -> ChunkState {
    use ChunkState::*;

    match curr_state {
        Empty => {
            if status.is_within_load { Loading } else { Empty }
        }
        Loading => {
            if status.loaded { Loaded } else { curr_state }
        }
        Loaded => {
            if status.is_within_render { ToDraw } 
            else { 
                if status.is_within_load { curr_state } else { Empty }
            }
        },
        ToDraw => {
            if status.is_within_render { 
                if status.needs_rebuild { Loaded } else { 
                    if status.mesh_built { Drawn } else { curr_state }
                }
            } else {
                Loaded
            }
        },
        Drawn => {
            if status.is_within_render { 
                if status.needs_rebuild { ToDraw } else { curr_state }
            } else {
                Loaded
            }
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkTransition {
	EmptyToLoading,
	LoadingToLoaded,
    LoadedToEmpty,
    LoadedToToDraw,
    ToDrawToLoaded,
    ToDrawToDrawn,
    DrawnToToDraw,
    DrawnToLoaded,
}

pub fn get_chunk_transition(from: ChunkState, to: ChunkState) -> Option<ChunkTransition> {
    use ChunkState::*;
    use ChunkTransition::*;

    match (from, to) {
        (Empty, Loading) => Some(EmptyToLoading),
        (Loading, Loaded) => Some(LoadingToLoaded),
        (Loaded, Empty) => Some(LoadedToEmpty),
        (Loaded, ToDraw) => Some(LoadedToToDraw),
        (ToDraw, Loaded) => Some(ToDrawToLoaded),
        (ToDraw, Drawn) => Some(ToDrawToDrawn),
        (Drawn, ToDraw) => Some(DrawnToToDraw),
        (Drawn, Loaded) => Some(DrawnToLoaded),
        _ => None,
    }
}

