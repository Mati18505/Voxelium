use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_common_assets::yaml::YamlAssetPlugin;

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath, Debug)]
pub struct Config {
    column_height: u32,
    render_distance: u32,
    seed: i32,
    use_threading: bool,
    block_ray_cast_increment: f32,

    world_scale: u32,
} 

impl Default for Config {
    fn default() -> Self {
        Self {
            column_height: 8,
            render_distance: 10,
            seed: 0,
            use_threading: false,
            block_ray_cast_increment: 0.5,
            
            world_scale: 1,
        }
    }
}