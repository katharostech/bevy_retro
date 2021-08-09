use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render2::camera::{
        DepthCalculation, OrthographicCameraBundle, OrthographicProjection, ScalingMode,
    },
};
use bevy_retrograde::prelude::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::new()
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
    const CAMERA_HEIGHT: f32 = 300.0; // The camera height in retro-resolution pixels
    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            // Note that the scale is half of the height
            scale: CAMERA_HEIGHT / 2.0,
            // This makes sure that the in-game height of the camera stays the same, while the width
            // adjusts automatically based on the aspect ratio
            scaling_mode: ScalingMode::FixedVertical,
            // This is the depth mode that should be used for 2D
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
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
                color: Color::RED,
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
