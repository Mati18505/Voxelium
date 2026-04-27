const CHUNK_SIZE: u32 = 16;

// block_sides
const UP: u32 = 0;
const DOWN: u32 = 1;
const LEFT: u32 = 2;
const RIGHT: u32 = 3;
const FRONT: u32 = 4;
const BACK: u32 = 5;

// UVs
const TOP_LEFT: u32 = 0;
const TOP_RIGHT: u32 = 1;
const BOTTOM_LEFT: u32 = 2;
const BOTTOM_RIGHT: u32 = 3;

fn block_side_to_normal(block_side: u32) -> vec3<f32> {
    switch (block_side) {
        case UP:    { return vec3<f32>(0.0, 1.0, 0.0); }
        case DOWN:  { return vec3<f32>(0.0, -1.0, 0.0); }
        case LEFT:  { return vec3<f32>(-1.0, 0.0, 0.0); }
        case RIGHT: { return vec3<f32>(1.0, 0.0, 0.0); }
        case FRONT: { return vec3<f32>(0.0, 0.0, 1.0); }
        case BACK:  { return vec3<f32>(0.0, 0.0, -1.0); }
        default:    { return vec3<f32>(0.0, 1.0, 0.0); }
    }
}

fn uv_to_vec(uv: u32) -> vec2<f32> {
    switch (uv) {
        case TOP_LEFT:    { return vec2<f32>(0.0, 1.0); }
        case TOP_RIGHT:  { return vec2<f32>(1.0, 1.0); }
        case BOTTOM_LEFT:  { return vec2<f32>(0.0, 0.0); }
        case BOTTOM_RIGHT: { return vec2<f32>(1.0, 0.0); }
        default:    { return vec2<f32>(0.0, 0.0); }
    }
}

fn pos_from_index(index: u32) -> vec3<f32> {
    // Legal range of vertex pos (chunks are connected, so connecting vertex adds one).
    let size = CHUNK_SIZE + 1;

    let z = index / (size * size);
    let rem = index % (size * size);
    let y = rem / size;
    let x = rem % size;

    return vec3<f32>(f32(x), f32(y), f32(z));
}

struct VertexData {
    pos_index: u32,
    block_side: u32,
    uv: u32,
    storage_index: u32,
}

fn unpack_vertex_data(packed: u32) -> VertexData {
    const POS_MASK: u32 = (1 << 13) - 1;
    const NORMAL_MASK: u32 = (1 << 3)  - 1;
    const UV_MASK: u32 = (1 << 2)  - 1;
    const SI_MASK: u32 = (1 << 8)  - 1;

    var data: VertexData;
    data.pos_index  =  packed        & POS_MASK;
    data.block_side     = (packed >> 13) & NORMAL_MASK;
    data.uv         = (packed >> 16) & UV_MASK;
    data.storage_index = (packed >> 18) & SI_MASK;

    return data;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) storage_index: u32,
};

fn calculate_light_influence(normal: vec3<f32>) -> f32 {
    let light_dir = normalize(vec3(0.5, 1.0, 0.3));

    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = 0.2;

    return ambient + diffuse;
}
