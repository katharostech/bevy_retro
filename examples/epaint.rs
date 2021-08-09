//! Epaint rendering can be useful for debug rendering. For example, you could use it to render the
//! path calculated by your pathfinding algorithm while you play the game, so that you can see where
//! you AI is trying to go, etc.
//!
//! The epaint integration is very primitive. The API has not yet be polished, but it should still
//! be functional enough to get basic epaint shapes rendering. Also, note that the egui shapes are
//! rendered at the screen resolution, not in the retro, low-resolution.
//!
//! Text is not supported yet.

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_retrograde::prelude::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Epaint Demo".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(move_diamond.system())
        .run();
}

// Marker component for the diamond we draw
struct Diamond;

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn_bundle(RetroCameraBundle {
        camera: RetroCamera {
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn a circle at the center of the world
    commands.spawn_bundle(ShapeBundle {
        shape: Shape::circle_filled(epaint::emath::pos2(0., 0.), 5., epaint::Color32::BLUE),
        ..Default::default()
    });

    // Spawn a rectangle behind the circle
    commands.spawn_bundle(ShapeBundle {
        shape: Shape::rect_filled(
            epaint::emath::Rect {
                min: epaint::pos2(-20., -20.),
                max: epaint::pos2(20., 20.),
            },
            1.,
            epaint::Color32::RED,
        ),
        transform: Transform::from_xyz(0., 0., -1.),
        ..Default::default()
    });

    // Draw a diamond around both
    commands
        .spawn_bundle(ShapeBundle {
            shape: Shape::closed_line(
                vec![
                    epaint::pos2(0., -40.),
                    epaint::pos2(40., 0.),
                    epaint::pos2(0., 40.),
                    epaint::pos2(-40., 0.),
                ],
                (1., epaint::Color32::GREEN),
            ),
            ..Default::default()
        })
        // Add our marker component so the `move_diamond` system can find it
        .insert(Diamond);
}

fn move_diamond(mut diamonds: Query<&mut Transform, With<Diamond>>, time: Res<Time>) {
    for mut transform in diamonds.iter_mut() {
        transform.scale = Vec3::splat((time.seconds_since_startup() as f32).sin() + 0.5);
        transform.rotation = Quat::from_axis_angle(
            Vec3::Z,
            (time.seconds_since_startup() as f32).sin() * 2. * PI,
        );
    }
}
