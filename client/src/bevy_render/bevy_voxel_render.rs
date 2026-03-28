use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

use crate::chunk_mesh_builder::meshers::ATTRIBUTE_PACKED_DATA;

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
    const VS_ASSET_PATH: &str = "shaders/voxel-vs.wgsl";
    const FS_ASSET_PATH: &str = "shaders/textured_cube.wgsl";
}

impl Material for TexturedCubeMaterial {
    fn vertex_shader() -> ShaderRef {
        Self::VS_ASSET_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        Self::FS_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_common(descriptor, layout)
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ColoredCubeMaterial {
    #[texture(0, dimension = "2d")]
    #[sampler(1)]
    pub color_palette: Handle<Image>,
}

impl ColoredCubeMaterial {
    const VS_ASSET_PATH: &str = "shaders/voxel-vs.wgsl";
    const FS_ASSET_PATH: &str = "shaders/colored_cube.wgsl";
}

impl Material for ColoredCubeMaterial {
    fn vertex_shader() -> ShaderRef {
        Self::VS_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        Self::FS_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_common(descriptor, layout)
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CutoutTexturedCubeMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl CutoutTexturedCubeMaterial {
    const VS_ASSET_PATH: &str = "shaders/voxel-vs.wgsl";
    const FS_ASSET_PATH: &str = "shaders/cutout_textured_cube.wgsl";
}

impl Material for CutoutTexturedCubeMaterial {
    fn vertex_shader() -> ShaderRef {
        Self::VS_ASSET_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        Self::FS_ASSET_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_common(descriptor, layout)
    }
}

fn specialize_common(
    descriptor: &mut RenderPipelineDescriptor,
    layout: &MeshVertexBufferLayoutRef,
) -> Result<(), SpecializedMeshPipelineError> {
    let vertex_layout = layout.0.get_layout(&[
        ATTRIBUTE_PACKED_DATA.at_shader_location(0),
    ])?;
    descriptor.vertex.buffers = vec![vertex_layout];
    Ok(())
}
