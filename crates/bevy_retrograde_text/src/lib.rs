//! Bevy Retrograde text rendering plugin

#![allow(clippy::type_complexity)]

use bevy::{asset::AssetStage, ecs::component::ComponentDescriptor, prelude::*};

#[doc(hidden)]
pub mod prelude {
    pub use crate::assets::*;
    pub use crate::components::*;
    pub use crate::RetroTextPlugin;
}

mod assets;

mod components;

pub(crate) mod bdf;

mod systems;
pub use systems::rasterize_text_block;
use systems::*;

use prelude::*;

/// The bevy stage the [`RetroTextPlugin`] runs its systems in
#[derive(StageLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct RetroTextStage;

/// Text rendering plugin for Bevy Retrograde
pub struct RetroTextPlugin;

impl Plugin for RetroTextPlugin {
    fn build(&self, app: &mut App) {
        app
            // use sparce storage for marker components
            .register_component(ComponentDescriptor::new::<TextNeedsUpdate>(
                bevy::ecs::component::StorageType::SparseSet,
            ))
            // Add our font asset
            .add_asset::<Font>()
            // Add our font asset loader
            .add_asset_loader(FontLoader)
            // Add our font rendering system
            .add_stage_before(
                // We have to run before assets are uploaded to prevent frame delays on text updates
                AssetStage::LoadAssets,
                RetroTextStage,
                SystemStage::single(font_rendering.system()),
            );
    }
}
