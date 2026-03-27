//#define DEBUG_NORMALS

#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
}

#import "shaders/common.wgsl"::{normal_to_vec, pos_from_index, unpack_vertex_data}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) @interpolate(flat) packed: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) normal: vec3<f32>,
    @location(1) storage_index: u32,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let vertex = unpack_vertex_data(in.packed);
    let position = pos_from_index(vertex.pos_index) - 0.5;
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(in.instance_index),
        vec4<f32>(position, 1.0),
    );
    out.storage_index = vertex.storage_index;
    out.normal = normal_to_vec(vertex.normal);
    return out;
}


@group(#{MATERIAL_BIND_GROUP}) @binding(0) var color_palette: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var color_palette_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3(0.5, 1.0, 0.3));

    let diffuse = max(dot(mesh.normal, light_dir), 0.0);
    let ambient = 0.2;

    let color_sample = f32(mesh.storage_index) / 256.0;
    let base_color = textureSample(color_palette, color_palette_sampler, vec2(color_sample, 0.5));

#ifdef DEBUG_COLOR_SAMPLE
    return vec4<f32>(color_sample, 0.0, 0.0, 0.0);
#else ifdef DEBUG_NORMALS
    return vec4<f32>(mesh.normal, 0.0);
#else ifdef DEBUG_DIFFUSE
    return vec4<f32>(diffuse, 0.0, 0.0, 0.0);
#else
    return base_color * (ambient + diffuse);
#endif
}
