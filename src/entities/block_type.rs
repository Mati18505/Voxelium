#[derive(Debug, Clone, PartialEq)]
pub struct BlockType {
    name: String,
    affect_raycast: bool,
}

impl BlockType {
    pub fn new(name: &str, affect_raycast: bool) -> Self {
        BlockType {
            name: name.to_owned(),
            affect_raycast,
        }
    }
}

impl Default for BlockType {
    fn default() -> Self {
        Self::new("unnamed", true)
    }
}