use std::fmt::{self, Debug};

use cgmath::Vector3;
use dot_vox::{DotVoxData, Model, Rotation, SceneNode, Voxel};
use thiserror::Error;

type FilePath = String;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Invalid vox file error: {0}")]
    InvalidFileError(String),
}

/// Represents one MagicaVoxel model.
#[derive(Clone, PartialEq)]
pub struct VoxModel {
    /// Global model position in world space.
    /// Represents the center point in the model.
    /// Coordinates are in voxel units.
    pub global_position: Vector3<i32>,

    /// The global dimensions of the model in voxels.
    /// Calculated as: `rotation * size`. May contain negative components, depending on rotation.
    pub global_size: Vector3<f32>,

    /// The local dimensions of the model in voxels. (width, height, depth)
    pub size: Vector3<u32>,

    /// Points at element {0, 0, 0}, which is at the bottom left corner.
    /// In world space.
    pub min_corner: Vector3<i32>,

    /// Points which is at the top right corner.
    /// In world space.
    pub max_corner: Vector3<i32>,

    /// The voxels to be displayed.
    pub voxels: Vec<Voxel>,
}

impl Debug for VoxModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxModel")
            .field("global_position", &self.global_position)
            .field("global_size", &self.global_size)
            .field("size", &self.size)
            .field("voxels len", &self.voxels.len())
            .finish()
    }
}

pub fn import(bytes: &[u8]) -> Result<Vec<VoxModel>, ImportError> {
    let vox_data =
        dot_vox::load_bytes(bytes).map_err(|err| ImportError::InvalidFileError(err.to_string()))?;

    let mut models = Vec::default();

    iterate_vox_data(&vox_data, |model, global_position, orientation| {
        //conversion to Vector3<i32> is required, because orientation might negate the
        // sign of the size components
        let orientation_matrix = glam::Mat3::from_cols_array_2d(&orientation.to_cols_array_2d());
        let orientation_matrix = from_glam_to_cgmath_matrix(orientation_matrix);
        let size_vec = Vector3::new(
            model.size.x as f32,
            model.size.y as f32,
            model.size.z as f32,
        );

        let global_position = *global_position;
        let global_size: Vector3<f32> = orientation_matrix * size_vec;
        let size = model.size;
        let size = Vector3::new(size.x, size.y, size.z);

        let half = (size.map(|v| v as i32) / 2);
        let min_corner = global_position - half;
        let max_corner = global_position + half;

        assert_size_consistent(max_corner, min_corner, size);

        models.push(VoxModel {
            global_position,
            global_size,
            size,
            min_corner,
            max_corner,
            voxels: model.voxels.clone(),
        });
    });

    Ok(models)
}

fn iterate_vox_data(
    vox_data: &DotVoxData,
    mut func: impl FnMut(&Model, &Vector3<i32>, &Rotation),
) -> Result<(), ImportError> {
    if vox_data.scenes.len() == 0 {
        let zero = Vector3::new(0, 0, 0);
        let identity = Rotation::IDENTITY;

        for model in &vox_data.models {
            func(&model, &zero, &identity);
        }

        Ok(())
    } else {
        iterate_vox_tree(vox_data, func)
    }
}

fn iterate_vox_tree(
    vox_tree: &DotVoxData,
    mut func: impl FnMut(&Model, &Vector3<i32>, &Rotation),
) -> Result<(), ImportError> {
    use ImportError::*;

    match &vox_tree.scenes[0] {
        SceneNode::Transform {
            attributes: _,
            frames: _,
            child,
            layer_id: _,
        } => iterate_vox_tree_inner(
            vox_tree,
            *child,
            Vector3::new(0, 0, 0),
            Rotation::IDENTITY,
            &mut func,
        ),
        _ => Err(InvalidFileError(
            "The root node for a magicka voxel DAG should be a Transform node!".to_string(),
        )),
    }
}

fn parse_translation_delta(t: &str) -> Result<Vec<i32>, ImportError> {
    t.split_whitespace()
        .map(|x| {
            x.parse::<i32>()
                .map_err(|err| ImportError::InvalidFileError(format!("Not an integer: {err}")))
        })
        .collect()
}

fn parse_rotation(r: &str) -> Result<Rotation, ImportError> {
    Ok(Rotation::from_byte(r.parse().map_err(|err| {
        ImportError::InvalidFileError(format!(
            "Expected valid u8 byte to parse rotation matrix: {err}"
        ))
    })?))
}

fn iterate_vox_tree_inner(
    vox_tree: &DotVoxData,
    current_node: u32,
    translation: Vector3<i32>,
    rotation: Rotation,
    func: &mut impl FnMut(&Model, &Vector3<i32>, &Rotation),
) -> Result<(), ImportError> {
    use ImportError::*;

    match &vox_tree.scenes[current_node as usize] {
        SceneNode::Transform {
            attributes: _,
            frames,
            child,
            layer_id: _,
        } => {
            // In case of a Transform node, the potential translation and rotation is added
            // to the global transform to all of the nodes children nodes
            let translation = match frames[0].attributes.get("_t") {
                Some(t) => {
                    let translation_delta = parse_translation_delta(t)?;

                    if translation_delta.len() != 3 {
                        return Err(InvalidFileError(
                            "Translation data should have 3 values!".to_string(),
                        ));
                    }

                    translation
                        + Vector3::new(
                            translation_delta[0],
                            translation_delta[1],
                            translation_delta[2],
                        )
                }
                None => translation,
            };

            let rotation = match frames[0].attributes.get("_r") {
                Some(r) => parse_rotation(r)?,
                None => Rotation::IDENTITY,
            };

            iterate_vox_tree_inner(vox_tree, *child, translation, rotation, func)?;
        }
        SceneNode::Group {
            attributes: _,
            children,
        } => {
            // in case the current node is a group, the index variable stores the current
            // child index
            for child_node in children {
                iterate_vox_tree_inner(vox_tree, *child_node, translation, rotation, func)?;
            }
        }
        SceneNode::Shape {
            attributes: _,
            models,
        } => {
            // in case the current node is a shape: it's a leaf node and it contains
            // models(voxel arrays)
            for model in models {
                func(
                    &vox_tree.models[model.model_id as usize],
                    &translation,
                    &rotation,
                );
            }
        }
    }

    Ok(())
}

fn from_glam_to_cgmath_matrix(matrix: glam::Mat3) -> cgmath::Matrix3<f32> {
    let cols = matrix.to_cols_array_2d();

    cgmath::Matrix3::from_cols(
        Vector3::new(cols[0][0], cols[0][1], cols[0][2]),
        Vector3::new(cols[1][0], cols[1][1], cols[1][2]),
        Vector3::new(cols[2][0], cols[2][1], cols[2][2]),
    )
}

fn vector3_geq(a: Vector3<i32>, b: Vector3<i32>) -> bool {
    a.x >= b.x && a.y >= b.y && a.z >= b.z
}

fn assert_size_consistent(max_corner: Vector3<i32>, min_corner: Vector3<i32>, size: Vector3<u32>) {
    assert!(vector3_geq(max_corner, min_corner), "max_corner must be >= min_corner element-wise");

    let diff = max_corner - min_corner;
    let diff_u32 = Vector3::new(diff.x as u32, diff.y as u32, diff.z as u32);

    assert!(diff_u32 == size, "Difference between corners must equal size");
}

