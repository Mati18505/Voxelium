//#define DEBUG_NORMALS
#import "shaders/common.wgsl"::{VertexOutput, calculate_light_influence}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var color_palette: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var color_palette_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let light_influence = calculate_light_influence(mesh.normal);
    let color_sample = f32(mesh.storage_index) / 256.0;
    let base_color = textureSample(color_palette, color_palette_sampler, vec2(color_sample, 0.5));

#ifdef DEBUG_COLOR_SAMPLE
    return vec4<f32>(color_sample, 0.0, 0.0, 0.0);
#else ifdef DEBUG_NORMALS
    return vec4<f32>(mesh.normal, 0.0);
#else ifdef DEBUG_DIFFUSE
    return vec4<f32>(diffuse, 0.0, 0.0, 0.0);
#else
    return base_color * light_influence;
#endif
}