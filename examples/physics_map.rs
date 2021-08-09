//! Physics in Bevy Retrograde currently just leverages the heron crate for almost
//! everything. The only difference is the use of the `TesselatedCollider` component that can be
//! used to create a convex hull collision shape from a sprite image.

use bevy_retrograde::core::image::DynamicImage;

use bevy_retrograde::core::image::GenericImageView;

use bevy::{core::FixedTimestep, prelude::*};
use bevy_retrograde::prelude::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Physics Map".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(update_map_collisions.system())
        .add_stage(
            "game_stage",
            SystemStage::parallel().with_run_criteria(FixedTimestep::step(0.015)),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    commands.insert_resource(Gravity::from(Vec3::new(0., 9.8 * 16., 0.)));

    // Spawn the camera
    commands.spawn_bundle(RetroCameraBundle {
        camera: RetroCamera {
            size: CameraSize::FixedHeight(160),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        transform: Transform::from_xyz(0., 0., 0.),
        ..Default::default()
    });

    // Spawn the map
    commands.spawn().insert_bundle(LdtkMapBundle {
        map: asset_server.load("maps/physicsDemoMap.ldtk"),
        ..Default::default()
    });

    let radish_images = [
        asset_server.load("redRadish.png"),
        asset_server.load("blueRadish.png"),
        asset_server.load("yellowRadish.png"),
    ];

    // Spawn bouncy radishes
    for y in 0..=2 {
        for x in -10..=10 {
            let sprite_image = radish_images[((x as i32).abs() % 3) as usize].clone();
            commands
                .spawn_bundle(SpriteBundle {
                    image: sprite_image.clone(),
                    sprite: Sprite {
                        pixel_perfect: false,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(x as f32 * 12., -80. - y as f32 * 20., 0.),
                    ..Default::default()
                })
                .insert(TesselatedCollider {
                    image: sprite_image,
                    tesselator_config: TesselatedColliderConfig {
                        // We want the collision shape for the player to be highly accurate
                        vertice_separation: 0.,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                // The player is also a dynamic body with rotations locked
                .insert(RigidBody::Dynamic)
                // WARNING: Rotations are not rendered in Bevy Retrograde yet, so if you don't lock the
                // rotation of dynamic bodies, the sprite will _look_ un-rotated, but the physics engine
                // will calculate it like it _is_ rotated.
                .insert(RotationConstraints::lock())
                // Disable friction and bounciness
                .insert(PhysicMaterial {
                    friction: 0.2,
                    restitution: 1.,
                    ..Default::default()
                })
                // Set the player speed to 0 initially
                .insert(Velocity::from_linear(Vec3::default()))
                .insert(Player);
        }
    }
}

struct MapLayerLoaded;
/// This system will go through each layer in spawned maps and generate a collision shape for each tile
fn update_map_collisions(
    mut commands: Commands,
    map_layers: Query<(Entity, &LdtkMapLayer, &Handle<Image>), Without<MapLayerLoaded>>,
    image_assets: Res<Assets<Image>>,
) {
    for (layer_ent, map_layer, image_handle) in map_layers.iter() {
        // ( which should be fixed eventually by rust-analyzer )
        let map_layer: &LdtkMapLayer = map_layer;

        let image = if let Some(image) = image_assets.get(image_handle) {
            image
        } else {
            continue;
        };

        // Get the tile size of the map
        let tile_size = map_layer.layer_instance.__grid_size as u32;

        let mut layer_commands = commands.entity(layer_ent);

        // For every tile grid
        for tile_x in 0u32..map_layer.layer_instance.__c_wid as u32 {
            for tile_y in 0u32..map_layer.layer_instance.__c_hei as u32 {
                // Get the tile image
                let tile_img = image
                    .view(tile_x * tile_size, tile_y * tile_size, tile_size, tile_size)
                    .to_image();

                // Try to generate a convex collision mesh from the tile
                let mesh = create_convex_collider(
                    DynamicImage::ImageRgba8(tile_img),
                    &TesselatedColliderConfig {
                        // The maximum accuracy for collision mesh generation
                        vertice_separation: 0.,
                        ..Default::default()
                    },
                );

                // If mesh generation was successful ( wouldn't be fore empty tiles, etc. )
                if let Some(mesh) = mesh {
                    // Spawn a collider as a child of the map layer
                    layer_commands.with_children(|layer| {
                        layer.spawn().insert_bundle((
                            mesh,
                            Transform::from_xyz(
                                (tile_x * tile_size + tile_size / 2) as f32,
                                (tile_y * tile_size + tile_size / 2) as f32,
                                0.,
                            ),
                            GlobalTransform::default(),
                        ));
                    });
                }
            }
        }

        layer_commands
            // Make layer a static body
            .insert(RigidBody::Static)
            // Mark as loaded
            .insert(MapLayerLoaded);
    }
}

struct Player;
