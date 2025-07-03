#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Empty,
    Generated,

    ToDraw,
    Drawn,
}

pub trait State {
}