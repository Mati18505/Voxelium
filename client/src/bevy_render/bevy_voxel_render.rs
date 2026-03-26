use bevy::{mesh::MeshVertexBufferLayoutRef, pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, render::render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError}, shader::ShaderRef};

pub struct VoxelRenderPlugin;
impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TexturedCubeMaterial>::default())
            .add_plugins(MaterialPlugin::<ColoredCubeMaterial>::default())
            .add_plugins(MaterialPlugin::<CutoutTexturedCubeMaterial>::default());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TexturedCubeMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl TexturedCubeMaterial {
    const SHADER_ASSET_PATH: &str = "shaders/textured_cube.wgsl";
}

impl Material for TexturedCubeMaterial {
    fn fragment_shader() -> ShaderRef {
        Self::SHADER_ASSET_PATH.into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ColoredCubeMaterial {
    #[texture(0, dimension = "2d")]
    #[sampler(1)]
    pub color_palette: Handle<Image>,
}

impl ColoredCubeMaterial {
    const SHADER_ASSET_PATH: &str = "shaders/colored_cube.wgsl";
}

impl Material for ColoredCubeMaterial {
    fn fragment_shader() -> ShaderRef {
        Self::SHADER_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_UV_1.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CutoutTexturedCubeMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl CutoutTexturedCubeMaterial {
    const SHADER_ASSET_PATH: &str = "shaders/cutout_textured_cube.wgsl";
}

impl Material for CutoutTexturedCubeMaterial {
    fn fragment_shader() -> ShaderRef {
        Self::SHADER_ASSET_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }
}
