use bevy::{core::FixedTimestep, prelude::*};
use bevy_retrograde::prelude::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Physics Character".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_stage(
            "game_stage",
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.015))
                .with_system(move_player.system()),
        )
        .run();
}

struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the camera
    commands.spawn_bundle(RetroCameraBundle {
        camera: RetroCamera {
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        transform: Transform::from_xyz(0., -50., 0.),
        ..Default::default()
    });

    // Load our images
    let block = asset_server.load("block.png");
    let triangle = asset_server.load("triangle.png");
    let red_radish = asset_server.load("redRadish.png");

    // Spawn a collider block that will just sit there and be an obstacle
    commands
        // First we spawn a sprite bundle like normal
        .spawn_bundle(SpriteBundle {
            image: block.clone(),
            ..Default::default()
        })
        // Then we add a tesselated collider component. This will create a convex collision shape
        // from the provided image automatically.
        .insert(TesselatedCollider {
            // We want to use the same block we use for the visual for the collider shape
            image: block.clone(),
            ..Default::default()
        })
        // Make it a static body
        .insert(RigidBody::Static);

    // Spawn a couple more blocks at different positions
    commands
        .spawn_bundle(SpriteBundle {
            image: block.clone(),
            transform: Transform::from_xyz(200., -24., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            image: block.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Static);
    commands
        .spawn_bundle(SpriteBundle {
            image: block.clone(),
            transform: Transform::from_xyz(-200., -24., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            image: block.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Static);

    // Spawn a triangle obstacle
    commands
        .spawn_bundle(SpriteBundle {
            image: triangle.clone(),
            transform: Transform::from_xyz(-50., -60., 0.),
            ..Default::default()
        })
        .insert(RigidBody::Static)
        .with_children(|parent| {
            parent.spawn().insert_bundle((
                Transform::default(),
                GlobalTransform::default(),
                TesselatedCollider {
                    image: triangle,
                    // For this obstacle we provide a custom configuration for the tesselator
                    tesselator_config: TesselatedColliderConfig {
                        // This vertice separation value sets the closest that any two tesselated vertices
                        // are allowed to be to each-other. In other words, the higher this value, the less
                        // acurate your collision box will be, but less compute expensive the collisions
                        // will be.
                        //
                        // By setting the separation to 0., the collision shape should be as close as
                        // possible to the actual pixel shape.
                        //
                        // The default value is 10.
                        vertice_separation: 30.,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ));
        });

    // Spawn the player
    commands
        .spawn_bundle(SpriteBundle {
            image: red_radish.clone(),
            sprite: Sprite {
                pixel_perfect: false,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., -50., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            image: red_radish.clone(),
            tesselator_config: TesselatedColliderConfig {
                // We want the collision shape for the player to be highly accurate
                vertice_separation: 0.,
                ..Default::default()
            },
            ..Default::default()
        })
        // The player is also a dynamic body with rotations locked
        .insert(RigidBody::Dynamic)
        .insert(RotationConstraints::lock())
        // Disable friction and bounciness
        .insert(PhysicMaterial {
            friction: 0.,
            restitution: 0.,
            ..Default::default()
        })
        // Set the player speed to 0 initially
        .insert(Velocity::from_linear(Vec3::default()))
        .insert(Player);
}

/// Set's the player speed based on input from the keyboard arrow keys
fn move_player(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in query.iter_mut() {
        const SPEED: f32 = 30.;

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

        *velocity = Velocity::from_linear(direction);
    }
}
