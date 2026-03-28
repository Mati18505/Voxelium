//#define DEBUG_UV
#import "shaders/common.wgsl"::{VertexOutput}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var my_array_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var my_array_texture_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3(0.5, 1.0, 0.3));

    let diffuse = max(dot(mesh.normal, light_dir), 0.0);
    let ambient = 0.2;

    let base_color = textureSample(my_array_texture, my_array_texture_sampler, mesh.uv, mesh.storage_index);

    if base_color.a < 0.5 {
        discard;
    }

#ifdef DEBUG_COLOR_SAMPLE
    return vec4<f32>(color_sample, 0.0, 0.0, 0.0);
#else ifdef DEBUG_NORMALS
    return vec4<f32>(mesh.normal, 0.0);
#else ifdef DEBUG_UV
    return vec4<f32>(mesh.uv, 0.0, 0.0);
#else ifdef DEBUG_DIFFUSE
    return vec4<f32>(diffuse, 0.0, 0.0, 0.0);
#else
    return base_color * (ambient + diffuse);
#endif
}