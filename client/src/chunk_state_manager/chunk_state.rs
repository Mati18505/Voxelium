#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Empty,
    Loaded,
    ToDraw,
    Drawn,
}