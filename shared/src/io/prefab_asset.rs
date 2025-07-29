use std::collections::HashMap;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    log,
    prelude::*,
};
use thiserror::Error;

use crate::io::vox_importer::{self, VoxModel};
use crate::entities::*;
use crate::voxel_edits::voxel_ops;

pub struct PrefabLoaderPlugin;

impl Plugin for PrefabLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PrefabAsset>()
            .init_asset_loader::<PrefabAssetLoader>();
    }
}

#[derive(Asset, TypePath, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct PrefabAsset(Prefab);

impl From<PrefabAsset> for Prefab {
    fn from(asset: PrefabAsset) -> Self {
        asset.0
    }
}

#[derive(Default)]
pub struct PrefabAssetLoader;

#[derive(Debug, Error)]
pub enum PrefabAssetLoaderError {
    #[error("File load error: {0}")]
    FileError(#[from] std::io::Error),
    #[error("Importer error: {0}")]
    ImporterError(#[from] vox_importer::ImportError),
}

impl AssetLoader for PrefabAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["vox"]
    }

    type Asset = PrefabAsset;
    type Settings = ();
    type Error = PrefabAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::default();
        reader.read_to_end(&mut bytes).await?;

        let vox_models: Vec<VoxModel> = vox_importer::import(&bytes)?;
        let mut prefab = Prefab::default();

        for model in vox_models {
            for voxel in model.voxels {
                // log::info!("{:?}", &model);
                
                let voxel_pos = BlockPos::new(voxel.x as isize, voxel.y as isize, voxel.z as isize);

                prefab.add_voxel(Voxel { pos: voxel_pos, id: 1 });
            }
        }

        Ok(PrefabAsset(prefab))
    }
}