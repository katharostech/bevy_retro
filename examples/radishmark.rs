use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_retrograde::prelude::*;
use rand::{thread_rng, Rng};

struct RadishCounter {
    pub count: u128,
}

struct Radish {
    velocity: Vec2,
}

struct RadishImage([Handle<Image>; 3]);

impl FromWorld for RadishImage {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        RadishImage([
            asset_server.load("redRadish.png"),
            asset_server.load("yellowRadish.png"),
            asset_server.load("blueRadish.png"),
        ])
    }
}

const GAME_WIDTH: i32 = 300;
const GAME_HEIGHT: i32 = 300;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde RadishMark".to_string(),
            width: 600.,
            height: 600.,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(RadishCounter { count: 0 })
        .init_resource::<RadishImage>()
        .insert_resource(UiTree(raui::prelude::widget! {
            (ui::fps_counter)
        }))
        .add_startup_system(setup.system())
        .add_system(mouse_handler.system())
        .add_system(movement_system.system())
        .add_system(collision_system.system())
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn().insert_bundle(RetroCameraBundle {
        camera: RetroCamera {
            size: CameraSize::LetterBoxed {
                width: GAME_WIDTH as u32,
                height: GAME_HEIGHT as u32,
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn mouse_handler(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    radish: Res<RadishImage>,
    mut counter: ResMut<RadishCounter>,
) {
    let mut rng = thread_rng();
    if mouse_button_input.pressed(MouseButton::Left) {
        for count in 0..10 {
            counter.count += 1;
            let bird_z = (counter.count + count) as i32 % 1024;
            commands
                .spawn_bundle(SpriteBundle {
                    image: radish.0[rng.gen_range(0..3)].clone(),
                    transform: Transform::from_xyz(
                        rng.gen_range(0.0..=10.0),
                        rng.gen_range(0.0..=10.0),
                        bird_z as f32,
                    ),
                    ..Default::default()
                })
                .insert(Radish {
                    velocity: Vec2::new(rng.gen_range(1.0..=2.0), rng.gen_range(1.0..=2.0)),
                });
        }
    }
}

fn movement_system(mut radishes: Query<(&Radish, &mut Transform)>) {
    for (radish, mut transform) in radishes.iter_mut() {
        transform.translation.x += radish.velocity.x;
        transform.translation.y += radish.velocity.y;
    }
}

fn collision_system(mut bird_query: Query<(&mut Radish, &Transform)>) {
    for (mut radish, transform) in bird_query.iter_mut() {
        if transform.translation.x.abs() > (GAME_WIDTH / 2) as f32 {
            radish.velocity.x *= -1.0;
        }
        if transform.translation.y.abs() > (GAME_HEIGHT / 2) as f32 {
            radish.velocity.y *= -1.0;
        }
    }
}

mod ui {
    use super::RadishCounter;
    use bevy::{
        diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
        prelude::World,
    };
    use bevy_retrograde::ui::raui::prelude::*;

    pub fn fps_counter(ctx: WidgetContext) -> WidgetNode {
        let world = ctx.process_context.get_mut::<World>().unwrap();

        let fps = {
            let diagnostics = world.get_resource::<Diagnostics>().unwrap();
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .map(|x| x.average())
                .flatten()
                .unwrap_or(0f64)
        };
        let count = world.get_resource::<RadishCounter>().unwrap().count;

        let text = format!(
            "FPS:   {:.0} \n\
             Count: {}",
            fps, count
        );

        let text_box_props = Props::new(TextBoxProps {
            text,
            font: TextBoxFont {
                name: "cozette.bdf".into(),
                size: 1.,
            },
            width: TextBoxSizeValue::Exact(80.),
            ..Default::default()
        })
        .with(ContentBoxItemLayout {
            margin: Rect {
                top: 10.,
                left: 10.,
                ..Default::default()
            },
            align: Vec2 { x: 1.0, y: 0.0 },
            ..Default::default()
        });

        widget! {
            (content_box [
                (text_box: {text_box_props})
            ])
        }
    }
}
