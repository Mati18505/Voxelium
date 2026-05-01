use bevy::prelude::*;

use crate::{bevy_resources::RuntimeMaterial, bevy_types::GameResources};

use super::materials::*;
use super::material_handle::MaterialHandle;

pub struct VoxelMaterialsPlugin;
impl Plugin for VoxelMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TexturedCubeMaterial>::default())
            .add_plugins(MaterialPlugin::<ColoredCubeMaterial>::default())
            .add_plugins(MaterialPlugin::<CutoutTexturedCubeMaterial>::default())
            .add_systems(Update, build_materials.run_if(resource_changed::<GameResources>));
    }
}

#[derive(Resource, Debug)]
pub struct RenderMaterials(pub Vec<MaterialHandle>);

fn build_materials(
    mut render_materials: ResMut<RenderMaterials>,
    mut placeholder_materials: ResMut<Assets<StandardMaterial>>,
    mut textured_materials: ResMut<Assets<TexturedCubeMaterial>>,
    mut colored_materials: ResMut<Assets<ColoredCubeMaterial>>,
    mut cutout_materials: ResMut<Assets<CutoutTexturedCubeMaterial>>,
    game_resources: Res<GameResources>,
) {
    let runtime_materials = &game_resources.material_storage;
    let runtime_textures = &game_resources.texture_storage;

    for runtime_material in runtime_materials.iter() {
        let render_material = match runtime_material {
            RuntimeMaterial::Placeholder => {
                let placeholder = StandardMaterial {
                    base_color: Color::srgba(0.54, 0.0, 0.54, 1.0),
                    ..Default::default()
                };
                MaterialHandle::Placeholder(placeholder_materials.add(placeholder))
            }
            RuntimeMaterial::Textured(texture_id) => {
                let array_texture = runtime_textures
                    .get_by_id(*texture_id as usize)
                    .cloned()
                    .unwrap();

                MaterialHandle::Textured(textured_materials.add(TexturedCubeMaterial{ array_texture }))
            }
            RuntimeMaterial::Colored(texture_id) => {
                let color_palette = runtime_textures
                    .get_by_id(*texture_id as usize)
                    .cloned()
                    .unwrap();

                MaterialHandle::Colored(colored_materials.add(ColoredCubeMaterial{ color_palette }))
            }
            RuntimeMaterial::CutoutTextured(texture_id) => {
                let array_texture = runtime_textures
                    .get_by_id(*texture_id as usize)
                    .cloned()
                    .unwrap();

                MaterialHandle::CutoutTextured(cutout_materials.add(CutoutTexturedCubeMaterial{ array_texture }))
            }
        };

        render_materials.0.push(render_material);
    }
}
