//! Bevy Retrograde is an opinionated plugin pack for the [Bevy] game engine with tools to help you
//! make 2D games!
//!
//! Bevy Retrograde is not specific to pixel-art games, but it does include some features that would
//! be particularly useful for pixel games. The ultimate goal is to act as an extension to Bevy that
//! gives you common tools necessary to make a 2D game such as map loading, physics, UI, save-data,
//! etc. Not all of the features we want to add are implemented yet, but we will be expanding the
//! feature set as we developer our own game with it.
//!
//! [Bevy]: https://bevyengine.org
//!
//! # License
//!
//! Bevy Retrograde LDtk is licensed under the [Katharos License][k_license] which places certain
//! restrictions on what you are allowed to use it for. Please read and understand the terms before
//! using Bevy Retrograde for your project.
//!
//! [k_license]: https://github.com/katharostech/katharos-license
//!
//! # Development Status
//!
//! Bevy Retrograde is in early stages of development. The API is not stable and may change
//! dramatically at any time.
//!
//! [`bevy_webgl2`]: https://github.com/mrk-its/bevy_webgl2
//!
//! [bevy_renderer2]: https://github.com/bevyengine/bevy/discussions/2351
//!
//! See also [Supported Bevy Version](#supported-bevy-version) below.
//!
//! # Features & Examples
//!
//! Check out our [examples] list to see how to use each Bevy Retrograde feature:
//!
//! - Supports web and desktop out-of-the-box
//! - [LDtk](https://ldtk.io) map loading and rendering
//! - An integration with the [RAUI] UI library for building in-game user interfaces and HUD
//! - Physics and collision detection powered by [Heron] and [Rapier] with automatic generation of
//!   convex collision shapes from sprite images
//! - Text rendering of bitmap fonts in the BDF format
//! - A simple but effective sound playing API
//!
//! [examples]:
//! https://github.com/katharostech/bevy_retrograde/tree/master/examples#bevy-retro-examples
//!
//! [luminance]: https://github.com/phaazon/luminance-rs
//!
//! [RAUI]: https://raui-labs.github.io/raui/
//!
//! [Heron]: https://github.com/jcornaz/heron
//!
//! [Rapier]: https://rapier.rs/
//!
//! # Supported Bevy Version
//!
//! Bevy Retrograde currently works on the latest Bevy release and may or may not support Bevy
//! master as well. Bevy Retrograde will try to follow the latest Bevy release, but if there are
//! features introduced in Bevy master that we need, we may require Bevy master for a time until the
//! next Bevy release.
//!
//! **`Cargo.toml`:**
//!
//! ```toml
//!  # The default-features setting is optional, but can make build times faster if you are only
//!  # developing 2D games.
//! bevy = { version = "0.6", default-features = false }
//! bevy_retrograde = "0.3.0"
//! ```

/// Bevy Retrograde default plugins
pub struct RetroPlugins;

impl bevy::app::PluginGroup for RetroPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        // Add the plugins we need from Bevy
        bevy::PipelinedDefaultPlugins.build(group);

        #[cfg(feature = "audio")]
        group.add(audio::RetroAudioPlugin);

        #[cfg(feature = "ldtk")]
        group.add(ldtk::LdtkPlugin);

        #[cfg(feature = "text")]
        group.add(text::RetroTextPlugin);

        #[cfg(feature = "physics")]
        group.add(physics::RetroPhysicsPlugin);

        #[cfg(feature = "ui")]
        group.add(ui::RetroUiPlugin);
    }
}

/// Bevy Retrograde prelude
#[doc(hidden)]
pub mod prelude {
    pub use crate::*;
    pub use bevy_retrograde_macros::impl_deref;

    #[cfg(feature = "audio")]
    pub use bevy_retrograde_audio::*;

    #[cfg(feature = "text")]
    pub use bevy_retrograde_text::prelude::*;

    #[cfg(feature = "ldtk")]
    pub use bevy_retrograde_ldtk::*;

    #[cfg(feature = "ui")]
    pub use bevy_retrograde_ui::*;

    #[cfg(feature = "epaint")]
    pub use bevy_retrograde_epaint::prelude::*;

    #[cfg(feature = "physics")]
    pub use bevy_retrograde_physics::*;
}

#[cfg(feature = "re-export-bevy")]
pub use bevy;

pub use bevy_retrograde_macros::impl_deref;

#[cfg(feature = "audio")]
#[doc(inline)]
pub use bevy_retrograde_audio as audio;

#[cfg(feature = "text")]
#[doc(inline)]
pub use bevy_retrograde_text as text;

#[cfg(feature = "physics")]
#[doc(inline)]
pub use bevy_retrograde_physics as physics;

#[cfg(feature = "ldtk")]
pub use bevy_retrograde_ldtk as ldtk;

#[cfg(feature = "ui")]
#[doc(inline)]
pub use bevy_retrograde_ui as ui;

#[cfg(feature = "epaint")]
#[doc(inline)]
pub use bevy_retrograde_epaint as epaint;
