use crate::bevy_resources::TextureId;

#[derive(Debug, Clone)]
pub enum RuntimeMaterial {
    Placeholder,
    Textured(TextureId),
    Colored(TextureId),
    CutoutTextured(TextureId),
}
