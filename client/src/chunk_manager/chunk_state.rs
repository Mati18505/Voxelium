use crate::chunk_mesh_builder::ChunkMesh;

/// State of Chunk data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    /// The `Chunk` does not exist.
    Empty,
    /// The `Chunk` is in loader queue.
    Loading,
    /// The `Chunk` is fully loaded and stored in the world.
    Loaded,
}

/// External inputs for determining next `ChunkState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkDataStatus {
    /// Whether the chunk is within the current load range.
    pub is_within_load: bool,

    /// Whether the chunk has been fully loaded and is ready to use.
    /// This usually indicates that asynchronous loading (e.g. from disk or generator) has completed.
    pub loaded: bool,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkTransition {
    pub from: ChunkState,
    pub to: ChunkState,
}

impl ChunkState {
    pub fn get_next_chunk_state(&self, status: ChunkDataStatus) -> ChunkState {
        use ChunkState::*;

        let curr_state = self;

        if !status.is_within_load {
            return Empty;
        }

        match curr_state {
            Empty => {
                Loading
            }
            Loading => {
                if status.loaded {
                    Loaded
                } else {
                    curr_state
                }
            }
            Loaded => {
                curr_state
            }
        }
    }

    pub fn get_chunk_transition(&self, to: ChunkState) -> Option<ChunkTransition> {
        use ChunkState::*;
        
        let from = self;

        match (from, to) {
            (Empty, Loading)
            | (Loading, Empty)
            | (Loading, Loaded)
            | (Loaded, Empty) => Some(ChunkTransition{ from, to }),
            _ => None,
        }
    }
}