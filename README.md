# bevy_retro

[![Crates.io](https://img.shields.io/crates/v/bevy_retro.svg)](https://crates.io/crates/bevy_retro)
[![Docs.rs](https://docs.rs/bevy_retro/badge.svg)](https://docs.rs/bevy_retro)
[![Build Status](https://github.com/katharostech/bevy_retro/actions/workflows/rust.yaml/badge.svg)](https://github.com/katharostech/bevy_retro/actions/workflows/rust.yaml)
[![lines of code](https://tokei.rs/b1/github/katharostech/bevy_retro?category=code)](https://github.com/katharostech/bevy_retro)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

<div align="center">
    <em>( Screenshot of <a href="https://katharostech.com/post/bounty-bros-on-web">Bounty Bros.</a> game made with Bevy Retro and <a href="https://github.com/katharostech/skipngo">Skip'n Go</a> )</em>
</div>

![bounty bros game screenshot](./doc/bounty_bros.png)

[skipngo]:  https://github.com/katharostech/skipngo

Bevy Retro is a 2D, pixel-perfect renderer for [Bevy][__link0] that can target both web and desktop using OpenGL/WebGL.

Bevy Retro is focused on providing an easy and ergonomic way to write 2D, pixel-perfect games. Compared to the out-of-the-box Bevy setup, you do not have to work with a 3D scene to create 2D games. Sprites and their coordinates are based on pixel positions in a retro-resolution scene.

Bevy Retro replaces almost all of the out-of-the-box Bevy components and Bundles that you would normally use ( `Transform`, `Camera2DBundle`, etc. ) and comes with its own `Position`, `Camera`, `Image`, `Sprite`, etc. components and bundles. Bevy Retro tries to provide a focused 2D-centric experience on top of Bevy that helps take out some of the pitfalls and makes it easier to think about your game when all you need is 2D.

We want to provide a batteries-included plugin that comes with almost everything you need to make a 2D pixel game with Bevy including, collisions, sound, saving data, etc. While adding these features we will try to maintain full web compatibility, but it can’t be guaranteed that all features will be feasible to implement for web.

These extra features will be included as optional cargo features that can be disabled if not needed and, where applicable, may be packaged as separate Rust crates that can be used even if you don’t want to use the rest of Bevy Retro.


## License

Bevy Retro LDtk is licensed under the [Katharos License][__link1] which places certain restrictions on what you are allowed to use it for. Please read and understand the terms before using Bevy Retro for your project.


## Development Status

Bevy Retro is in early stages of development. The API is not stable, but there are not many large anticipated changes. Bevy Retro should be usable enough to use in your own projects if you are fine adapting to some API changes as they come.

See also [Supported Bevy Version](#supported-bevy-version) below.


## Features & Examples

Check out our [examples][__link2] list to see how to use each Bevy Retro feature:

 - Supports web and desktop out-of-the-box
 - Integer pixel coordinates
 - Supports sprites and sprite sheets
 - A super-simple hierarchy system
 - Scaled pixel-perfect rendering with three camera modes: fixed width, fixed height, and letter-boxed
 - [LDtk][__link3] map loading and rendering
 - An integration with the [RAUI][__link4] UI library for building in-game user interfaces and HUD
 - Pixel-perfect collision detection
 - Text rendering of BDF fonts
 - Custom shaders for post-processing, including a built-in CRT shader
 - Render hooks allowing you to drop down into raw [Luminance][__link5] calls for custom rendering


## Supported Bevy Version

Bevy Retro currently works on the latest Bevy release and *may* support Bevy master as well. Bevy Retro will try to follow the latest Bevy release, but if there are features introduced in Bevy master that we need, we may require Bevy master for a time until the next Bevy release.

When depending on the `bevy` crate, you must be sure to set `default-features` to `false` in your `Cargo.toml` so that the rendering types in `bevy` don’t conflict with the ones in `bevy_retro`.

**`Cargo.toml`:**


```toml
bevy = { version = "0.5", default-features = false }
bevy_retro = { git = "https://github.com/katharostech/bevy_retro.git" }
```


## Sample

Here’s a quick sample of what using Bevy Retro looks like:

**`main.rs`:**


```rust
use bevy::prelude::*;
use bevy_retro::prelude::*;

fn main() {
    App::build()
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .run();
}

struct Player;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    // Load our sprites
    let red_radish_image = asset_server.load("redRadish.png");
    let yellow_radish_image = asset_server.load("yellowRadish.png");
    let blue_radish_image = asset_server.load("blueRadish.png");

    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            // Set our camera to have a fixed height and an auto-resized width
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        position: Position::new(0, 0, 0),
        ..Default::default()
    });

    // Spawn a red radish
    let red_radish = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: red_radish_image,
            position: Position::new(0, 0, 0),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..Default::default()
            },
            ..Default::default()
        })
        // Add our player marker component so we can move it
        .insert(Player)
        .id();

    // Spawn a yellow radish
    let yellow_radish = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: yellow_radish_image,
            position: Position::new(-20, 0, 0),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    // Make the yellow radish a child of the red radish
    scene_graph
        .add_child(red_radish, yellow_radish)
        // This could fail if the child is an ancestor of the parent
        .unwrap();

    // Spawn a blue radish
    commands.spawn().insert_bundle(SpriteBundle {
        image: blue_radish_image,
        // Set the blue radish back a layer so that he shows up under the other two
        position: Position::new(-20, -20, -1),
        sprite: Sprite {
            flip_x: true,
            flip_y: false,
            ..Default::default()
        },
        ..Default::default()
    });
}
```



 [__link0]: https://bevyengine.org
 [__link1]: https://github.com/katharostech/katharos-license
 [__link2]: https://github.com/katharostech/bevy_retro/tree/master/examples#bevy-retro-examples
 [__link3]: https://ldtk.io
 [__link4]: https://raui-labs.github.io/raui/
 [__link5]: https://github.com/phaazon/luminance-rs

