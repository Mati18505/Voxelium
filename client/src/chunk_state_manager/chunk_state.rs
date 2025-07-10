#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Loaded,

    ToDraw,
    Drawn,
}

pub trait State {
}