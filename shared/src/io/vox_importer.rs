use cgmath::Vector3;
use dot_vox::{DotVoxData, Model, Rotation, SceneNode};
use thiserror::Error;

type FilePath = String;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ImportError {
    #[error("Invalid vox file error: {0}")]
    InvalidFileError(String),

    #[error("Vox file loader error: {0}, while loading file: {1}")]
    FileLoadError(String, FilePath),
}

fn iterate_vox_tree(
    vox_tree: &DotVoxData,
    mut fun: impl FnMut(&Model, &Vector3<i32>, &Rotation),
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
            &mut fun,
        ),
        _ => Err(InvalidFileError("The root node for a magicka voxel DAG should be a Transform node!".to_string())),
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

#[allow(unused)]
pub fn import(file: &str) -> Result<(), ImportError> {
    let vox_tree = dot_vox::load(file)
        .map_err(|err| ImportError::FileLoadError(err.to_string(), file.to_string()))?;

    iterate_vox_tree(&vox_tree, |model, position, orientation| {
        //conversion to Vector3<i32> is required, because orientation might negate the
        // sign of the size components
        let orientation_matrix = glam::Mat3::from_cols_array_2d(&orientation.to_cols_array_2d());
        let orientation_matrix = from_glam_to_cgmath_matrix(orientation_matrix);
        let size_vec = Vector3::new(
            model.size.x as f32,
            model.size.y as f32,
            model.size.z as f32,
        );

        let model_size: Vector3<f32> = orientation_matrix * size_vec;
        let position = position.map(|e| e as f32);
        let min_corner = position - model_size / 2.0;
        let max_corner = position + model_size / 2.0;

        // The global position points to the middle of the model, the element at
        // [0][0][0] is at the bottom left corner
        println!("model size: {model_size:?} position of element[0][0][0]: {min_corner:?}",);
    });

    Ok(())

    // for each block in model:
        // if chunk does not exist - create it
        // voxel_ops::set_block()
}
