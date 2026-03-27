use bevy::{mesh::MeshVertexBufferLayoutRef, pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, render::render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError}, shader::ShaderRef};

use crate::chunk_mesh_builder::meshers::{ATTRIBUTE_BLOCK_IN_CHUNK_POS, ATTRIBUTE_BLOCK_SIDE, ATTRIBUTE_STORAGE_INDEX, ATTRIBUTE_UV};

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
    fn vertex_shader() -> ShaderRef {
        Self::SHADER_ASSET_PATH.into()
    }

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
            ATTRIBUTE_BLOCK_IN_CHUNK_POS.at_shader_location(0),
            ATTRIBUTE_BLOCK_SIDE.at_shader_location(1),
            ATTRIBUTE_UV.at_shader_location(2),
            ATTRIBUTE_STORAGE_INDEX.at_shader_location(3),
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
