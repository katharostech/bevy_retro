//! Asset types

use crate::prelude::*;
use bevy::prelude::*;

mod image;
pub use self::image::*;

/// Add asset types and asset loader to the app builder
pub(crate) fn add_assets(app: &mut AppBuilder) {
    app.add_asset::<Image>()
        .init_asset_loader::<ImageLoader>()
        .add_asset::<SpriteSheet>();
}
