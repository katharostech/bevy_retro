use bevy::{core::FixedTimestep, prelude::*};
use bevy_retrograde::prelude::*;

#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Sprite Sheet".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.015))
                .with_system(animate_sprite.system()),
        )
        .run();
}

/// This component helps us keep track of which frame we're on for our sprite
struct SpriteAnimFrame(usize);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprite_sheet_assets: ResMut<Assets<RetroSpriteSheet>>,
) {
    // Spawn sprite
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite_bundle: SpriteBundle {
                image: asset_server.load("redRadishSheet.png"),
                ..Default::default()
            },
            sprite_sheet: sprite_sheet_assets.add(RetroSpriteSheet {
                grid_size: UVec2::splat(16),
                tile_index: 4,
            }),
        })
        .insert(SpriteAnimFrame(0));

    // Spawn camera
    commands.spawn().insert_bundle(RetroCameraBundle {
        camera: RetroCamera {
            size: CameraSize::FixedHeight(40),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn animate_sprite(
    // Keep track if the frame number
    mut frame: Local<u8>,
    mut query: Query<(&Handle<RetroSpriteSheet>, &mut SpriteAnimFrame), With<Handle<Image>>>,
    mut sprite_sheet_assets: ResMut<Assets<RetroSpriteSheet>>,
) {
    // Increment frame number
    *frame = frame.wrapping_add(1);

    let frames = [4, 5, 6, 7];

    // Play the next animation frame every 10 frames
    if *frame % 10 == 0 {
        *frame = 0;
        for (sprite_sheet_handle, mut frame) in query.iter_mut() {
            if let Some(sprite_sheet) = sprite_sheet_assets.get_mut(sprite_sheet_handle) {
                frame.0 = frame.0.wrapping_add(1);
                sprite_sheet.tile_index = frames[frame.0 % frames.len()];
            }
        }
    }
}
