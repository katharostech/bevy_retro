use bevy::{core::FixedTimestep, prelude::*, utils::HashSet};
use bevy_retro::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

struct RadishImages {
    collided: Handle<Image>,
    uncollided: Handle<Image>,
}

impl FromWorld for RadishImages {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        RadishImages {
            collided: asset_server.load("redRadish.png"),
            uncollided: asset_server.load("blueRadish.png"),
        }
    }
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Collisions".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .init_resource::<RadishImages>()
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.05))
                .with_system(move_player.system())
                .with_system(collision_detection.system()),
        )
        .run();
}

struct Player;

fn setup(mut commands: Commands, radish_images: Res<RadishImages>) {
    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        position: Position::new(0, 0, 0),
        ..Default::default()
    });

    // Spawn a radish for the player
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: radish_images.uncollided.clone(),
            position: Position::new(0, 0, -1),
            ..Default::default()
        })
        .insert(Player)
        .id();

    // Spawn some radishes that just sit there
    for (x, y) in &[(-20, 0), (-20, -20), (20, 20), (20, 0)] {
        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                image: radish_images.uncollided.clone(),
                position: Position::new(*x, *y, 0),
                ..Default::default()
            })
            .id();
    }
}

fn move_player(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Position, With<Player>>) {
    for mut pos in query.iter_mut() {
        const SPEED: i32 = 1;

        let mut direction = IVec3::new(0, 0, 0);

        if keyboard_input.pressed(KeyCode::Left) {
            direction += IVec3::new(-SPEED, 0, 0);
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += IVec3::new(SPEED, 0, 0);
        }

        if keyboard_input.pressed(KeyCode::Up) {
            direction += IVec3::new(0, -SPEED, 0);
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction += IVec3::new(0, SPEED, 0);
        }

        if direction != IVec3::new(0, 0, 0) {
            **pos += direction;
        }
    }
}

fn collision_detection(
    // We will need to read and write to the radish entities at different stages of the collision
    // detection so we create a query set to enforce that don't borrow the reading and writing
    // queries at the same time.
    mut radishes: QuerySet<(
        Query<(Entity, &Handle<Image>)>,
        Query<(Entity, &mut Handle<Image>)>,
    )>,
    mut collisions: PixelCollisions,
    image_assets: Res<Assets<Image>>,
    radish_images: Res<RadishImages>,
) {
    // Make sure collision positions are synchronized
    collisions.sync_positions();

    // Create list of colliding radishes
    let mut colliding_radishes = HashSet::default();

    // Create list of radish pairs we've already checked
    let mut checked_radishes = HashSet::default();

    for (radish1, radish1_col) in radishes.q0().iter() {
        // Get the collision image for radish 1
        let radish1_col = if let Some(col) = image_assets.get(radish1_col) {
            col
        } else {
            continue;
        };

        for (radish2, radish2_col) in radishes.q0().iter() {
            // Skip if radish two is the same radish as radish 1 or if we've already checked this
            // pair
            if radish1 == radish2
                || checked_radishes.contains(&(radish1, radish2))
                || checked_radishes.contains(&(radish2, radish1))
            {
                continue;
            }

            // Get collision image for radish 2
            let radish2_col = if let Some(col) = image_assets.get(radish2_col) {
                col
            } else {
                continue;
            };

            // If they are colliding
            if collisions.collides_with(radish1, radish1_col, radish2, radish2_col) {
                // Add them to the colliding radish list
                colliding_radishes.insert(radish1);
                colliding_radishes.insert(radish2);
            }

            // Add this pair to the list of radishes we've checked
            checked_radishes.insert((radish1, radish2));
        }
    }

    // Make all colliding radishes red and non-colliding radishes blue
    for (radish, mut image) in radishes.q1_mut().iter_mut() {
        if colliding_radishes.contains(&radish) {
            *image = radish_images.collided.clone();
        } else {
            *image = radish_images.uncollided.clone();
        }
    }
}
