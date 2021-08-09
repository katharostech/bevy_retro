use bevy::{core::FixedTimestep, prelude::*};
use bevy_retrograde::prelude::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Post-Processing".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.015))
                .with_system(move_player.system()),
        )
        .run();
}

struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load our sprites
    let red_radish_image = asset_server.load("redRadish.png");
    let yellow_radish_image = asset_server.load("yellowRadish.png");
    let blue_radish_image = asset_server.load("blueRadish.png");

    // Spawn the camera
    commands.spawn().insert_bundle(RetroCameraBundle {
        camera: RetroCamera {
            // Set our camera to have a fixed height and an auto-resized width
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            // Add a custom shader
            custom_shader: Some(
                // Here we use the built-in CRT shader. We could also alternatively provide our own
                // shader string. See `src/shaders/crt_shader.glsl` for an example.
                CrtShader {
                    // We can change the default settings here
                    scan_line_opacity: 0.3,
                    ..Default::default()
                }
                .get_shader(),
            ),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn a red radish
    let red_radish = commands
        .spawn_bundle(SpriteBundle {
            image: red_radish_image,
            transform: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        })
        // Add our player marker component so we can move it
        .insert(Player)
        .id();

    // Spawn a yellow radish
    let yellow_radish = commands
        .spawn_bundle(SpriteBundle {
            image: yellow_radish_image,
            transform: Transform::from_xyz(-20., 0., 0.),
            ..Default::default()
        })
        .id();

    // Make the yellow radish a child of the red radish
    commands.entity(red_radish).push_children(&[yellow_radish]);

    // Spawn a blue radish
    commands.spawn().insert_bundle(SpriteBundle {
        image: blue_radish_image,
        // Set the blue radish back a layer so that he shows up under the other two
        transform: Transform::from_xyz(-20., -20., -1.),
        sprite: Sprite {
            flip_x: true,
            flip_y: false,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in query.iter_mut() {
        const SPEED: f32 = 1.;

        let mut direction = Vec3::new(0., 0., 0.);

        if keyboard_input.pressed(KeyCode::Left) {
            direction += Vec3::new(-SPEED, 0., 0.);
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += Vec3::new(SPEED, 0., 0.);
        }

        if keyboard_input.pressed(KeyCode::Up) {
            direction += Vec3::new(0., -SPEED, 0.);
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction += Vec3::new(0., SPEED, 0.);
        }

        if direction != Vec3::new(0., 0., 0.) {
            transform.translation += direction;
        }
    }
}
