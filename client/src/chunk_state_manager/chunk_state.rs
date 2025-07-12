#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Empty,
    Loaded,
    ToDraw,
    Drawn,
}

pub fn get_next_chunk_state(
    curr_state: ChunkState,
    is_within_render: bool,
    is_within_load: bool,
    mesh_built: bool,
) -> ChunkState {
    use ChunkState::*;

    match curr_state {
        Empty => {
            if is_within_load { Loaded } else { Empty }
        }
        Loaded => {
            if is_within_render { ToDraw } 
            else { 
                if is_within_load { curr_state } else { Empty }
            }
        },
        ToDraw => {
            if is_within_render { 
                if mesh_built { Drawn } else { curr_state }
            } else {
                Loaded
            }
        },
        Drawn => {
            if is_within_render { curr_state } else { Loaded }
        },
    }
}

pub enum ChunkTransition {
	EmptyToLoaded,
    LoadedToEmpty,
    LoadedToToDraw,
    ToDrawToLoaded,
    ToDrawToDrawn,
    DrawnToToDraw,
    DrawnToLoaded,
}

pub fn get_chunk_transition(from: ChunkState, to: ChunkState) -> ChunkTransition {
    assert_ne!(from, to);

    use ChunkState::*;
    use ChunkTransition::*;

    match (from, to) {
        (Empty, Loaded) => EmptyToLoaded,
        (Loaded, Empty) => LoadedToEmpty,
        (Loaded, ToDraw) => LoadedToToDraw,
        (ToDraw, Loaded) => ToDrawToLoaded,
        (ToDraw, Drawn) => ToDrawToDrawn,
        (Drawn, ToDraw) => DrawnToToDraw,
        (Drawn, Loaded) => DrawnToLoaded,
        _ => unreachable!(),
    }
}
