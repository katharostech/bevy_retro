//! Bevy Retrograde audio plugin
//!
//! Bevy Retrograde attempts to provide an _extremely_ simple, yet still effective API for playing audio
//! in games using [Kira]. If more control is desired you may want to look into [`bevy_kira_audio`]
//! which grants more control over audio playback.
//!
//! [`bevy_kira_audio`]: https://github.com/NiklasEi/bevy_kira_audio
//!
//! [Kira]: https://docs.rs/kira

use bevy::prelude::*;

pub use kira;

mod assets;
pub use assets::*;

mod components;
pub use components::*;

mod systems;
pub(crate) use systems::*;

/// Bevy Retrograde audio plugin
#[derive(Default)]
pub struct RetroAudioPlugin;

impl Plugin for RetroAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add audio manager resource
            .insert_non_send_resource(AudioManager::default())
            .add_event::<SoundEvent>();

        // Add asssets and systems
        add_assets(app);
        add_systems(app);
    }
}

pub use events::*;
mod events {
    use super::*;

    /// A sound event used to control the sound playback system
    #[doc(hidden)]
    #[derive(Debug, Clone)]
    #[allow(clippy::large_enum_variant)]
    pub enum SoundEvent {
        CreateSound(Handle<SoundData>, Sound),
        PlaySound(Sound, PlaySoundSettings),
        PauseSound(Sound, PauseSoundSettings),
        ResumeSound(Sound, ResumeSoundSettings),
        StopSound(Sound, StopSoundSettings),
    }
}
