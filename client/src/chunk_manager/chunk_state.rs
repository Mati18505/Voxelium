#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Generated,

    ToDraw,
    Drawn,
}

pub trait State {
}