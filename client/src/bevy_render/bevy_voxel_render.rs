use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}};

const SHADER_ASSET_PATH: &str = "shaders/array_texture.wgsl";

pub struct VoxelRenderPlugin;
impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            MaterialPlugin::<VoxelMaterial>::default(),
        );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
