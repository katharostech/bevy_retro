use bevy::{
    prelude::*,
    render2::camera::{
        DepthCalculation, OrthographicCameraBundle, OrthographicProjection, ScalingMode,
    },
    sprite2::PipelinedSpriteBundle,
};
use bevy_retrograde::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Physics Character".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(move_player)
        .run();
}

struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the camera
    const CAMERA_HEIGHT: f32 = 150.0;
    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            scale: CAMERA_HEIGHT / 2.0,
            scaling_mode: ScalingMode::FixedVertical,
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        },
        transform: Transform::from_xyz(0., 75., 0.),
        ..OrthographicCameraBundle::new_2d()
    });

    // Load our images
    let block = asset_server.load("block.png");
    let triangle = asset_server.load("triangle.png");
    let red_radish = asset_server.load("redRadish.png");

    // Spawn a collider block that will just sit there and be an obstacle
    commands
        // First we spawn a sprite bundle like normal
        .spawn_bundle(PipelinedSpriteBundle {
            texture: block.clone(),
            ..Default::default()
        })
        // Then we add a tesselated collider component. This will create a convex collision shape
        // from the provided image automatically.
        .insert(TesselatedCollider {
            // We want to use the same block we use for the visual for the collider shape
            texture: block.clone(),
            ..Default::default()
        })
        // Make it a static body
        .insert(RigidBody::Static);

    // Spawn a couple more blocks at different positions
    commands
        .spawn_bundle(PipelinedSpriteBundle {
            texture: block.clone(),
            transform: Transform::from_xyz(200., 24., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            texture: block.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Static);
    commands
        .spawn_bundle(PipelinedSpriteBundle {
            texture: block.clone(),
            transform: Transform::from_xyz(-200., 24., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            texture: block.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Static);

    // Spawn a triangle obstacle
    commands
        .spawn_bundle(PipelinedSpriteBundle {
            texture: triangle.clone(),
            transform: Transform::from_xyz(-50., 60., 0.),
            ..Default::default()
        })
        .insert(RigidBody::Static)
        .with_children(|parent| {
            parent.spawn().insert_bundle((
                Transform::default(),
                GlobalTransform::default(),
                TesselatedCollider {
                    texture: triangle,
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
        .spawn_bundle(PipelinedSpriteBundle {
            texture: red_radish.clone(),
            transform: Transform::from_xyz(0., 50., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            texture: red_radish.clone(),
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
fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
    time: Res<Time>,
) {
    for mut velocity in query.iter_mut() {
        let speed: f32 = 10.0 * time.delta().as_millis() as f32;

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

        *velocity = Velocity::from_linear(direction);
    }
}
