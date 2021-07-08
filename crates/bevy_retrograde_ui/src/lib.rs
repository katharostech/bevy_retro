//! Bevy Retrograde UI plugin

use bevy::prelude::*;

use bevy_retrograde_core::prelude::AppBuilderRenderHookExt;

mod resources;
pub use resources::*;

mod render_hook;
use render_hook::UiRenderHook;

pub(crate) mod interaction;

pub use raui;

/// Text rendering plugin for Bevy Retrograde
pub struct RetroUiPlugin;

impl Plugin for RetroUiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // Add the UI tree resource
            .init_resource::<UiTree>()
            .add_render_hook::<UiRenderHook>();
    }
}
