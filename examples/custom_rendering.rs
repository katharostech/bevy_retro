//! This example demonstrates how to use `RenderHook`'s to do custom rendering. This is a very
//! advanced example. Writing a custom render hook is only necessary if the objects you are
//! rendering cannot be represented with sprite images.
//!
//! The rendering types and modules will be moved around soon so this is somewhat work-in-progres.
//!
//! Bevy Retrograde uses [luminance](https://github.com/phaazon/luminance-rs) for rendering, so custom
//! rendering in Bevy Retrograde must also use luminance.

use bevy::prelude::*;
use bevy_retrograde::{
    core::{
        graphics::*,
        luminance::{
            self, context::GraphicsContext, pipeline::PipelineState, render_state::RenderState,
            shader::Uniform, tess::Mode, Semantics, UniformInterface, Vertex,
        },
    },
    prelude::*,
};

/// This is our triangle component. Our custom renderer will render all entities with a Triangle
/// component as a colored triangle.
struct Triangle {
    scale: f32,
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Custom Rendering".into(),
            ..Default::default()
        })
        // Here we add a custom `TriangleRenderHook` that we will setup to render objects with the
        // `Triangle` component.
        .add_plugins(RetroPlugins)
        .add_render_hook::<TriangleRenderHook>()
        .add_startup_system(setup.system())
        .add_system(move_triangle.system())
        .run();
}

/// In our setup function we will spawn a triangle
fn setup(mut commands: Commands) {
    // Spawn a camera
    commands.spawn().insert_bundle(RetroCameraBundle::default());

    // Spawn a triangle
    commands.spawn().insert_bundle((
        Triangle { scale: 0.5 },
        Transform::default(),
        GlobalTransform::default(),
    ));
}

/// Scale our triangle up and down and move it left and right
fn move_triangle(time: Res<Time>, mut query: Query<(&mut Triangle, &mut Transform)>) {
    for (mut tri, mut transform) in query.iter_mut() {
        tri.scale = time.seconds_since_startup().sin() as f32;
        transform.translation.x = (time.seconds_since_startup().sin() * 400.).round() as f32;
    }
}

/// And we will create our render hook struct that will do our custom rendering. This struct
/// contains the persistant graphics objects that it will use while rendering.
struct TriangleRenderHook {
    tri_program: Program<VertexSemantics, (), Uniforms>,
    tri_tess: Tess<Vertex>,

    current_triangle_batch: Option<Vec<Entity>>,
}

/// To make our render hook actually do rendering, we must implement [`RenderHook`] for it.
impl RenderHook for TriangleRenderHook {
    /// The init function is repsonsible for doing any one-time initialization and returning the new
    /// render hook trait object.
    fn init(_window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook>
    where
        Self: Sized,
    {
        let tri_program = surface
            .new_shader_program::<VertexSemantics, (), Uniforms>()
            .from_strings(VERT_SHADER, None, None, FRAG_SHADER)
            .expect("program creation")
            .ignore_warnings();

        let tri_tess = surface
            .new_tess()
            .set_vertices(TRI_VERTICES)
            .set_mode(Mode::Triangle)
            .build()
            .unwrap();

        Box::new(Self {
            tri_program,
            tri_tess,
            current_triangle_batch: None,
        })
    }

    /// The prepare function is responsible for returning a list of objects that will be rendered by
    /// this hook in this frame and returning what depth in the scene they are at so that the
    /// renderer can do depth sorting.
    ///
    /// This function should not do any rendering yet.
    fn prepare(
        &mut self,
        world: &mut World,
        _surface: &mut Surface,
        // This is a hash map that maps [`Handle<Image>`] to luminance GPU textures that can be
        // added to shader uniforms
        _texture_cache: &mut TextureCache,
        _frame_context: &FrameContext,
    ) -> Vec<RenderHookRenderableHandle> {
        // We create a query for all of our triangles in our scene
        let mut triangles = world.query::<(Entity, &Triangle, &GlobalTransform)>();

        // Start a list of triangle entities
        let mut triangle_batch = Vec::new();
        let mut triangle_depths = Vec::new();
        for (ent, _, transform) in triangles.iter(world) {
            triangle_batch.push(ent);
            triangle_depths.push(transform.translation.z);
        }

        // Set our current batch
        self.current_triangle_batch = Some(triangle_batch);

        // Convert our triangle list to a vector of renderables and return it
        self.current_triangle_batch
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, e)| RenderHookRenderableHandle {
                // We keep track of the renderable using its index in our triangle batch
                identifier: i,
                // Our triangles are not transparent ( this value is used during depth sorting )
                is_transparent: false,
                // We just render at the center of the world depth-wise
                depth: triangle_depths[i],
                // We can specify the entity here to sort by which order entities were spawned when
                // the depth and transparency is identical
                entity: Some(*e),
            })
            .collect()
    }

    /// The render function is called, possibly multiple times, after calling the prepare function
    /// and is where the actual rendering happens. When called, the render function should render
    /// only the renderables that are passed in in the `renderables` argument so that the depth is
    /// handled properly.
    fn render(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        _texture_cache: &mut TextureCache,
        _frame_context: &FrameContext,
        // This is the framebuffer that we should render to
        target_framebuffer: &SceneFramebuffer,
        // This is the list of renderables that we should render
        renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            current_triangle_batch,
            tri_program,
            tri_tess,
            ..
        } = self;

        let mut triangles = world.query::<(Entity, &Triangle, &GlobalTransform)>();

        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the target framebuffer
                &target_framebuffer,
                &PipelineState::default().enable_clear_color(false),
                |_, mut shading_gate| {
                    shading_gate.shade(tri_program, |mut interface, uniforms, mut render_gate| {
                        for renderable in renderables {
                            // Get the triangle for this renderable
                            let (_, tri, transform) = triangles
                                .get(
                                    world,
                                    *current_triangle_batch
                                        .as_ref()
                                        .unwrap()
                                        .get(renderable.identifier)
                                        .unwrap(),
                                )
                                .unwrap();
                            let pos = transform.translation;

                            // Set the triangle uniforms
                            interface.set(
                                &uniforms.tri_pos,
                                [pos.x as f32, pos.y as f32, pos.z as f32],
                            );
                            interface.set(&uniforms.tri_scale, tri.scale);

                            // Render the triangle
                            render_gate.render(&RenderState::default(), |mut tess_gate| {
                                tess_gate.render(&*tri_tess)
                            })?
                        }

                        Ok(())
                    })
                },
            )
            .assume()
            .into_result()
            .expect("Could not render");

        *current_triangle_batch = None;
    }
}

//
// Below here we define the core types used for doing the triangle rendering with Luminance. See the
// [Luminance Tutorial](https://rust-tutorials.github.io/learn-luminance/) for more info on how that
// works.
//

// Shaders must be written in GLES 1.0 format
const FRAG_SHADER: &str = r#"
varying vec4 v_color;

void main() {
  gl_FragColor = v_color;
}
"#;

const VERT_SHADER: &str = r#"
attribute vec2 pos;
attribute vec4 color;

uniform vec3 tri_pos;
uniform float tri_scale;

varying vec4 v_color;

void main() {
  v_color = color;
  gl_Position = vec4(pos * tri_scale + tri_pos.xy / 1024., tri_pos.z / 1024., 1.);
}
"#;

#[derive(Debug, UniformInterface)]
struct Uniforms {
    #[uniform(unbound)]
    tri_pos: Uniform<[f32; 3]>,
    #[uniform(unbound)]
    tri_scale: Uniform<f32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    // - Reference vertex positions with the "co" variable in vertex shaders.
    // - The underlying representation is [f32; 2], which is a vec2 in GLSL.
    // - The wrapper type you can use to handle such a semantics is VertexPosition.
    #[sem(name = "pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    // - Reference vertex colors with the "color" variable in vertex shaders.
    // - The underlying representation is [u8; 3], which is a uvec3 in GLSL.
    // - The wrapper type you can use to handle such a semantics is VertexColor.
    #[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexColor")]
    Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct Vertex {
    pos: VertexPosition,
    // Here, we can use the special normalized = <bool> construct to state whether we want integral
    // vertex attributes to be available as normalized floats in the shaders, when fetching them from
    // the vertex buffers. If you set it to "false" or ignore it, you will get non-normalized integer
    // values (i.e. value ranging from 0 to 255 for u8, for instance).
    #[vertex(normalized = "true")]
    rgb: VertexColor,
}

const TRI_VERTICES: [Vertex; 3] = [
    Vertex::new(
        VertexPosition::new([0.5, -0.5]),
        VertexColor::new([0., 1., 0., 1.]),
    ),
    Vertex::new(
        VertexPosition::new([0.0, 0.5]),
        VertexColor::new([0., 0., 1., 1.]),
    ),
    Vertex::new(
        VertexPosition::new([-0.5, -0.5]),
        VertexColor::new([1., 0., 0., 1.]),
    ),
];
