use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_retrograde::prelude::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Text".into(),
            ..Default::default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(fps.system())
        .run();
}

struct Fps;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the camera
    commands.spawn().insert_bundle(RetroCameraBundle {
        camera: RetroCamera {
            size: CameraSize::FixedHeight(300),
            ..Default::default()
        },
        ..Default::default()
    });

    // Bevy Retrograde reads the BDF font format
    let font = asset_server.load("cozette.bdf");

    // Spawn a single line of text
    commands.spawn().insert_bundle(TextBundle {
        text: Text {
            text: "The Beginning".into(),
            ..Default::default()
        },
        font: font.clone(),
        transform: Transform::from_xyz(0., -110., 0.),
        ..Default::default()
    });

    let long_text = "Once upon a time in a galaxy far, far away, there \
    was a game engine, built on Bevy and forged in Rustiness.\n\nThat engine \
    needed text rendering and so here we are with lame filler verbiage.";

    // Spawn a multi-line text block that automatically wraps at a certain width
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                text: long_text.into(),
                color: Color::new(1., 0., 0., 1.),
            },
            font: font.clone(),
            ..Default::default()
        })
        .insert(TextBlock {
            width: 120,
            ..Default::default()
        });

    // Text will spawn similar to sprites with the center of the text box at the entities position
    // but this can be changed in the same way as sprites, by turning off centering in the Sprite
    // copmponent. Non-centered text will be spawned with the top-left of the text box at the
    // position.
    commands.spawn().insert_bundle(TextBundle {
        text: Text {
            text: "- The Sign Painter".into(),
            ..Default::default()
        },
        sprite: Sprite {
            centered: false,
            ..Default::default()
        },
        font: font.clone(),
        transform: Transform::from_xyz(0., 110., 0.),
        ..Default::default()
    });

    // Add a frames per second counter
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                text: "FPS:".into(),
                ..Default::default()
            },
            sprite: Sprite {
                centered: false,
                ..Default::default()
            },
            font: font.clone(),
            transform: Transform::from_xyz(-200., -110., 0.),
            ..Default::default()
        })
        .insert(Fps);
}

fn fps(mut query: Query<&mut Text, With<Fps>>, diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        for mut text in query.iter_mut() {
            if let Some(fps) = fps.average() {
                text.text = format!("FPS: {:.0}", fps);
            }
        }
    }
}
