#import "shaders/common.wgsl"::{normal_to_vec, pos_from_index, unpack_vertex_data, VertexOutput}
#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) @interpolate(flat) packed: u32,
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