#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Empty,
    Loading,
    Loaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkStatus {
    pub is_within_load: bool,
    pub loaded: bool,
}

pub fn get_next_chunk_state(curr_state: ChunkState, status: ChunkStatus) -> ChunkState {
    use ChunkState::*;

    match curr_state {
        Empty => {
            if status.is_within_load {
                Loading
            } else {
                Empty
            }
        }
        Loading => {
            if status.is_within_load {
                if status.loaded {
                    Loaded
                } else {
                    curr_state
                }
            } else {
                Empty
            }
        }
        Loaded => {
            if status.is_within_load {
                curr_state
            } else {
                Empty
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkTransition {
    EmptyToLoading,
    LoadingToEmpty,
    LoadingToLoaded,
    LoadedToEmpty,
}

pub fn get_chunk_transition(from: ChunkState, to: ChunkState) -> Option<ChunkTransition> {
    use ChunkState::*;
    use ChunkTransition::*;

    match (from, to) {
        (Empty, Loading) => Some(EmptyToLoading),
        (Loading, Empty) => Some(LoadingToEmpty),
        (Loading, Loaded) => Some(LoadingToLoaded),
        (Loaded, Empty) => Some(LoadedToEmpty),
        _ => None,
    }
}
