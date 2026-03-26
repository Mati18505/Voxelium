alias BlockSide = u32;
const UP: BlockSide = 0;
const DOWN: BlockSide = 1;
const LEFT: BlockSide = 2;
const RIGHT: BlockSide = 3;
const FRONT: BlockSide = 4;
const BACK: BlockSide = 5;

alias UV = u32;
const TOP_LEFT: UV = 0;
const TOP_RIGHT: UV = 1;
const BOTTOM_LEFT: UV = 2;
const BOTTOM_RIGHT: UV = 3;

fn normal_to_vec(n: BlockSide) -> vec3<f32> {
    switch (n) {
        case UP:    { return vec3<f32>(0.0, 1.0, 0.0); }
        case DOWN:  { return vec3<f32>(0.0, -1.0, 0.0); }
        case LEFT:  { return vec3<f32>(-1.0, 0.0, 0.0); }
        case RIGHT: { return vec3<f32>(1.0, 0.0, 0.0); }
        case FRONT: { return vec3<f32>(0.0, 0.0, 1.0); }
        case BACK:  { return vec3<f32>(0.0, 0.0, -1.0); }
        default:    { return vec3<f32>(0.0, 1.0, 0.0); }
    }
}

fn uv_to_vec(n: UV) -> vec2<f32> {
    switch (n) {
        case TOP_LEFT:    { return vec2<f32>(0.0, 0.0); }
        case TOP_RIGHT:  { return vec2<f32>(0.0, 1.0); }
        case BOTTOM_LEFT:  { return vec2<f32>(1.0, 0.0); }
        case BOTTOM_RIGHT: { return vec2<f32>(1.0, 1.0); }
        default:    { return vec2<f32>(0.0, 0.0); }
    }
}