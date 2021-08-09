use bevy::{
    prelude::*,
    render2::camera::{
        DepthCalculation, OrthographicCameraBundle, OrthographicProjection, ScalingMode,
    },
    sprite2::{PipelinedSpriteBundle, Sprite},
};
use bevy_retrograde::prelude::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Hello World".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(move_player.system())
        .run();
}

struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load our sprites
    let red_radish_image = asset_server.load("redRadish.png");
    let yellow_radish_image = asset_server.load("yellowRadish.png");
    let blue_radish_image = asset_server.load("blueRadish.png");

    // Spawn the camera
    const CAMERA_HEIGHT: f32 = 80.0; // The camera height in retro-resolution pixels
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

    // Spawn a red radish
    let red_radish = commands
        .spawn_bundle(PipelinedSpriteBundle {
            texture: red_radish_image,
            transform: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        })
        // Add our player marker component so we can move it
        .insert(Player)
        .id();

    // Spawn a yellow radish
    let yellow_radish = commands
        .spawn_bundle(PipelinedSpriteBundle {
            texture: yellow_radish_image,
            transform: Transform::from_xyz(-20., 0., 0.),
            sprite: Sprite {
                // Make him upside down 🙃
                flip_y: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    // Make the yellow radish a child of the red radish
    commands.entity(red_radish).push_children(&[yellow_radish]);

    // Spawn a blue radish
    commands.spawn_bundle(PipelinedSpriteBundle {
        texture: blue_radish_image,
        // Set the blue radish back a layer so that he shows up under the other two
        transform: Transform::from_xyz(-20.0, 20.0, 1.0),
        ..Default::default()
    });
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        let speed: f32 = 20.0 * time.delta_seconds();

        let mut direction = Vec3::new(0., 0., 0.);

        if keyboard_input.pressed(KeyCode::Left) {
            direction += Vec3::new(-speed, 0., 0.);
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += Vec3::new(speed, 0., 0.);
        }

        if keyboard_input.pressed(KeyCode::Up) {
            direction += Vec3::new(0., speed, 0.);
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction += Vec3::new(0., -speed, 0.);
        }

        if direction != Vec3::new(0., 0., 0.) {
            transform.translation += direction;
        }
    }
}
