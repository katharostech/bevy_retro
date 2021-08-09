use bevy::prelude::*;
use bevy_retrograde::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Audio".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut sound_controller: SoundController) {
    // Load the sound data
    let sound_data = asset_server.load("blink.ogg");

    // Create a sound from the sound data. Sounds can be played, paused, resumed, and stopped
    let sound = sound_controller.create_sound(&sound_data);

    // Play the sound
    sound_controller.play_sound(sound);

    // Load the music
    let music_data = asset_server.load("music.ogg");
    let music = sound_controller.create_sound(&music_data);

    // Play it on loop
    sound_controller.play_sound_with_settings(
        music,
        PlaySoundSettings::new().loop_start(LoopStart::Custom(0.)),
    )
}
