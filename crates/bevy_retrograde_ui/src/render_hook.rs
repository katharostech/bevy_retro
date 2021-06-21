use std::collections::HashMap;

use bevy::{
    asset::{AssetPath, HandleId, LoadState},
    core::Time,
    math::{Mat4, Vec3},
    prelude::{AssetServer, Assets, Handle, Mut, World},
    utils::HashSet,
};
use bevy_retrograde_core::{
    graphics::{
        FrameContext, Program, RenderHook, RenderHookRenderableHandle, SceneFramebuffer, Surface,
        Tess, TextureCache,
    },
    luminance::{
        self,
        blending::{Blending, Equation, Factor},
        context::GraphicsContext,
        face_culling::FaceCulling,
        pipeline::{PipelineState, TextureBinding},
        pixel::{NormRGBA8UI, NormUnsigned},
        render_state::RenderState,
        scissor::ScissorRegion,
        shader::Uniform,
        tess::View,
        texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Wrap},
        Semantics, UniformInterface, Vertex,
    },
    prelude::{Color, Image},
};
use bevy_retrograde_text::{prelude::*, rasterize_text_block};
use raui::{
    prelude::{Application, CoordsMapping, DefaultLayoutEngine, ProcessContext, Rect, Renderer},
    renderer::tesselate::{
        prelude::TesselateRenderer,
        tesselation::{Batch, Tesselation, TesselationVerticesFormat},
    },
};

use crate::{interaction::BevyInteractionsEngine, UiTree};

trait AssetPathExt {
    fn format_as_load_path(&self) -> String;
}

impl<'a> AssetPathExt for AssetPath<'a> {
    fn format_as_load_path(&self) -> String {
        self.path()
            .to_str()
            .expect("Only valid unicode paths are supported")
            .to_string()
            + &self
                .label()
                .map(|x| format!("#{}", x))
                .unwrap_or_else(|| String::from(""))
    }
}

/// The render hook responsible for rendering the UI
pub struct UiRenderHook {
    app: Application,
    current_ui_tesselation: Option<Tesselation>,
    text_tess: Tess<UiVert>,
    shader_program: Program<(), (), UiUniformInterface>,
    /// Cache of image handles that the UI is using
    ///
    /// This cache makes sure that the ref-count on the image assets doesn't drop to zero and cause
    /// the image to be un-loaded while the UI id depending on it
    image_cache: HashSet<Handle<Image>>,
    handle_to_path: HashMap<HandleId, String>,
    /// Cache of fonts that the UI is using
    font_cache: HashSet<Handle<Font>>,
    interactions: BevyInteractionsEngine,
    has_shown_clipping_warning: bool,
}

impl RenderHook for UiRenderHook {
    fn init(_window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook>
    where
        Self: Sized,
    {
        Box::new(Self {
            current_ui_tesselation: None,
            shader_program: surface
                .new_shader_program::<(), (), UiUniformInterface>()
                .from_strings(
                    include_str!("render_hook/ui.vert"),
                    None,
                    None,
                    include_str!("render_hook/ui.frag"),
                )
                .unwrap()
                .program,
            text_tess: surface
                .new_tess()
                .set_vertices(&QUAD_VERTS[..])
                .set_mode(luminance::tess::Mode::TriangleFan)
                .build()
                .unwrap(),

            // Font & Image handle cache
            font_cache: Default::default(),
            image_cache: Default::default(),
            handle_to_path: Default::default(),
            interactions: Default::default(),
            has_shown_clipping_warning: false,
            app: {
                let mut app = Application::new();
                app.setup(raui::core::widget::setup);
                app.setup(raui::material::setup);

                app
            },
        })
    }

    fn prepare(
        &mut self,
        world: &mut World,
        _surface: &mut Surface,
        texture_cache: &mut TextureCache,
        frame_context: &FrameContext,
    ) -> Vec<RenderHookRenderableHandle> {
        // Scope the borrow of the world and its resources
        let ui_tesselation = {
            // Update interactions
            self.interactions
                .update(world, frame_context.target_sizes.low);

            // Get our bevy resources from the world
            let delta_time = world.get_resource::<Time>().unwrap().delta_seconds();

            // Get the app from the world ( we will re-insert it when we are done processing the app )
            world.resource_scope(|world: &mut World, ui_tree: Mut<UiTree>| {
                // Update the widget tree if it has changed
                if ui_tree.is_changed() {
                    self.app.apply(ui_tree.0.clone());
                }

                // Update delta time
                self.app.animations_delta_time = delta_time;

                // Run forced_process so that UI components run every frame in more of an "immediate
                // mode" fashion.
                //
                // TODO: Maybe change this if it doesn't make sense
                self.app.forced_process_with_context(
                    // Add the Bevy world to the process context
                    ProcessContext::new().insert_mut(world),
                );

                self.app
                    .interact(&mut self.interactions)
                    .expect("Couldn't run UI interactions");
                self.app.consume_signals();

                // For now we don't do image atlases
                let atlases = HashMap::default();

                // Collect image sizes from the textures in the texture cache
                let image_sizes = texture_cache
                    .iter()
                    .filter_map(|(handle, texture)| {
                        let asset_path = self.handle_to_path.get(&handle.id)?;
                        let size = texture.size();
                        Some((
                            asset_path.clone(),
                            raui::prelude::Vec2 {
                                x: size[0] as f32,
                                y: size[1] as f32,
                            },
                        ))
                    })
                    .collect();

                // Get the coordinate mapping based on the size of the screen
                let coords_mapping = CoordsMapping::new(Rect {
                    left: 0.,
                    top: 0.,
                    right: frame_context.target_sizes.low.x as f32,
                    bottom: frame_context.target_sizes.low.y as f32,
                });

                // Calculate app layout
                self.app
                    .layout(&coords_mapping, &mut DefaultLayoutEngine)
                    .expect("Could not layout UI");

                // Tesselate the UI
                let ui_tesselation = TesselateRenderer::new(
                    TesselationVerticesFormat::Interleaved,
                    (),
                    &atlases,
                    &image_sizes,
                )
                .render(
                    &self.app.rendered_tree(),
                    &coords_mapping,
                    &self.app.layout_data(),
                )
                .expect("Could not tesselate UI");

                ui_tesselation
            })
        };

        // Store the UI tesselation in preparation for rendering
        self.current_ui_tesselation = Some(ui_tesselation);

        vec![
            // We only do one render pass so we create one renderable
            RenderHookRenderableHandle {
                identifier: 0,
                depth: f32::INFINITY, // We render on top of everything else
                is_transparent: true,
                entity: None,
            },
        ]
    }

    fn render(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        texture_cache: &mut TextureCache,
        frame_context: &FrameContext,
        target_framebuffer: &SceneFramebuffer,
        // We only have one renderable for everything so we don't need to read this
        _renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            current_ui_tesselation,
            shader_program,
            font_cache,
            image_cache,
            handle_to_path,
            text_tess,
            has_shown_clipping_warning,
            ..
        } = self;

        // Get world resources
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let font_assets = world.get_resource::<Assets<Font>>().unwrap();

        // Get the UI tesselation
        let ui_tesselation = current_ui_tesselation.take().unwrap();

        // Collect vertices
        let vertices = ui_tesselation
            .vertices
            .as_interleaved()
            .unwrap()
            .iter()
            .map(|vertice| UiVert {
                pos: VertexPosition::new([vertice.position.x.floor(), vertice.position.y.floor()]),
                uv: VertexUv::new([vertice.tex_coord.x, vertice.tex_coord.y]),
                color: VertexColor::new([
                    vertice.color.r,
                    vertice.color.g,
                    vertice.color.b,
                    vertice.color.a,
                ]),
            })
            .collect::<Vec<_>>();

        // Upload the vertices to the GPU
        let tess = surface
            .new_tess()
            .set_mode(luminance::tess::Mode::Triangle)
            .set_vertices(vertices)
            .set_indices(ui_tesselation.indices)
            .build()
            .unwrap();
        let batches = ui_tesselation.batches;

        // Create the render state
        let mut render_state = RenderState::default()
            .set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::SrcAlphaComplement,
            })
            .set_face_culling(Some(FaceCulling {
                order: luminance::face_culling::FaceCullingOrder::CW,
                mode: luminance::face_culling::FaceCullingMode::Back,
            }))
            .set_depth_test(None); // Disable depth test so the UI always renders on top

        // Get list of image handles used by the UI
        for image_path in batches.iter().filter_map(|x| match x {
            Batch::ImageTriangles(image, _) => Some(image),
            _ => None,
        }) {
            // Get the texture handle
            let texture_handle: Handle<Image> =
                asset_server.get_handle(HandleId::from(AssetPath::from(image_path.as_str())));

            // Map the handle ID to the handle path if necessary
            //
            // TODO: This is just waiting on this Bevy PR to be merged:
            // https://github.com/bevyengine/bevy/pull/1290
            handle_to_path
                .entry(texture_handle.id)
                .or_insert_with(|| image_path.clone());

            // Load the texture if loading has not started yet
            if let LoadState::NotLoaded = asset_server.get_load_state(&texture_handle) {
                asset_server.load::<Image, _>(image_path.as_str());
            }

            // Add the image to the image cache to keep the handle from getting dropped while the
            // UI is using it.
            image_cache.insert(texture_handle);

            // TODO: Images used by the UI aren't ever cleaned up. If the UI uses an image at some
            // point, we assume that it might at any time want to use it again so we avoid
            // re-loading the image by just not un-loading the image. This could be a problem for
            // some UIs. We should find a way to make this configurable somehow.
            // We have the same issue with the fonts below.
        }

        // Get list of font handles used by the UI
        for font_path in batches.iter().filter_map(|x| match x {
            Batch::ExternalText(_, batch) => Some(&batch.font),
            _ => None,
        }) {
            // Get the font handle
            let font_handle: Handle<Font> =
                asset_server.get_handle(HandleId::from(AssetPath::from(font_path.as_str())));

            // Load the font if loading has not started yet
            if let LoadState::NotLoaded = asset_server.get_load_state(&font_handle) {
                asset_server.load::<Font, _>(font_path.as_str());
            }

            font_cache.insert(font_handle);
        }

        // Rasterize text blocks to textures
        // TODO: Cache text block rasterizations and reuse if they haven't been changed
        let mut text_block_textures = HashMap::new();
        for (widget, batch) in batches.iter().filter_map(|x| match x {
            Batch::ExternalText(widget, batch) => Some((widget, batch)),
            _ => None,
        }) {
            // Get the font handle
            let font_handle: Handle<Font> =
                asset_server.get_handle(HandleId::from(AssetPath::from(batch.font.as_str())));
            // Load the font
            let font = if let Some(font) = font_assets.get(font_handle) {
                font
            } else {
                continue;
            };

            // Collect text info
            let text = Text {
                text: batch.text.clone(),
                color: Color {
                    r: batch.color.r,
                    g: batch.color.g,
                    b: batch.color.b,
                    a: batch.color.a,
                },
            };
            let text_block = TextBlock {
                width: batch.box_size.x.round() as u32,
                horizontal_align: match batch.horizontal_align {
                    raui::prelude::TextBoxHorizontalAlign::Left => TextHorizontalAlign::Left,
                    raui::prelude::TextBoxHorizontalAlign::Center => TextHorizontalAlign::Center,
                    raui::prelude::TextBoxHorizontalAlign::Right => TextHorizontalAlign::Right,
                },
                vertical_align: match batch.vertical_align {
                    raui::prelude::TextBoxVerticalAlign::Top => TextVerticalAlign::Top,
                    raui::prelude::TextBoxVerticalAlign::Middle => TextVerticalAlign::Middle,
                    raui::prelude::TextBoxVerticalAlign::Bottom => TextVerticalAlign::Bottom,
                },
                height: Some(batch.box_size.y.round() as u32),
            };

            // Rasterize the text block
            let image = rasterize_text_block(&text, font, Some(&text_block));

            // Get raw pixels
            let (sprite_width, sprite_height) = image.dimensions();
            let image_size = [sprite_width, sprite_height];
            let pixels = image.as_raw();

            // Upload the image to the GPU
            let mut texture = surface
                .new_texture::<Dim2, NormRGBA8UI>(image_size, 0, PIXELATED_SAMPLER)
                .unwrap();
            texture.upload_raw(GenMipmaps::No, pixels).unwrap();

            text_block_textures.insert(widget.clone(), texture);
        }

        // The stack of clipping regions applied by RAUI
        let mut clip_stack = Vec::new();

        // Do the render
        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the scene framebuffer
                &target_framebuffer,
                &PipelineState::default().enable_clear_color(false),
                |pipeline, mut shading_gate| {
                    shading_gate.shade(
                        shader_program,
                        |mut interface, uniforms, mut render_gate| {
                            // Set the target size uniform
                            let target_size = frame_context.target_sizes.low;
                            interface.set(
                                &uniforms.target_size,
                                [target_size.x as f32, target_size.y as f32],
                            );

                            for batch in batches {
                                match batch {
                                    Batch::ColoredTriangles(tris) => {
                                        // Set widget type uniform
                                        interface.set(&uniforms.widget_type, WIDGET_COLORED_TRIS);

                                        render_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(tess.view(tris).unwrap())
                                        })?;
                                    }
                                    Batch::ImageTriangles(texture_path, tris) => {
                                        let texture_handle = asset_server.get_handle(
                                            HandleId::from(AssetPath::from(texture_path.as_str())),
                                        );

                                        // Get the texture using the image handle
                                        let texture = if let Some(texture) =
                                            texture_cache.get_mut(&texture_handle)
                                        {
                                            texture
                                        } else {
                                            // Skip for this frame
                                            continue;
                                        };

                                        // Bind our texture
                                        let bound_texture = pipeline.bind_texture(texture).unwrap();

                                        // Set the texture uniforms
                                        interface.set(&uniforms.texture, bound_texture.binding());
                                        interface.set(&uniforms.widget_type, WIDGET_IMAGE_TRIS);

                                        // Render the block
                                        render_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(tess.view(tris).unwrap())
                                        })?;
                                    }
                                    Batch::ExternalText(widget, batch) => {
                                        // Get the texture
                                        let texture = if let Some(tex) =
                                            text_block_textures.get_mut(&widget)
                                        {
                                            tex
                                        } else {
                                            continue;
                                        };

                                        // Bind our texture
                                        let tex_size = texture.size();
                                        let bound_texture = pipeline.bind_texture(texture).unwrap();
                                        interface.set(&uniforms.widget_type, WIDGET_TEXT);

                                        let m = batch.matrix;

                                        // Set the text block transform
                                        interface.set(
                                            &uniforms.text_box_transform,
                                            [
                                                [m[0], m[4], m[8], m[12].round()],
                                                [m[1], m[5], m[9], m[13].round()],
                                                [m[2], m[6], m[10], m[14].round()],
                                                [m[3], m[7], m[11], m[15]],
                                            ],
                                        );
                                        // Set the text block size
                                        interface.set(
                                            &uniforms.text_box_size,
                                            [tex_size[0] as f32, tex_size[1] as f32],
                                        );

                                        // Set the texture uniform
                                        interface.set(&uniforms.texture, bound_texture.binding());

                                        // Render the block
                                        render_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(&*text_tess)
                                        })?;
                                    }
                                    Batch::FontTriangles(_, _, _) => {
                                        unimplemented!("Tesselated font rendering not implemented")
                                    }
                                    Batch::ClipPush(clip) => {
                                        // Calculate clipping rectangle x and y
                                        let matrix = Mat4::from_cols_array(&clip.matrix);

                                        // tl, tr, bl, br == top_left, top_right, bottom_left, bottom_right
                                        let tl = matrix.project_point3(Vec3::new(0.0, 0.0, 0.0));
                                        let tr = matrix.project_point3(Vec3::new(
                                            clip.box_size.x,
                                            0.0,
                                            0.0,
                                        ));
                                        let br = matrix.project_point3(Vec3::new(
                                            clip.box_size.x,
                                            clip.box_size.y,
                                            0.0,
                                        ));
                                        let bl = matrix.project_point3(Vec3::new(
                                            0.0,
                                            clip.box_size.y,
                                            0.0,
                                        ));
                                        let x1 = tl.x.min(tr.x).min(br.x).min(bl.x).round();
                                        let y1 = tl.y.min(tr.y).min(br.y).min(bl.y).round();
                                        let x2 = tl.x.max(tr.x).max(br.x).max(bl.x).round();
                                        let y2 = tl.y.max(tr.y).max(br.y).max(bl.y).round();
                                        let width = x2 - x1;
                                        let height = y2 - y1;

                                        // Set the clipping section for future renders
                                        if !*has_shown_clipping_warning {
                                            bevy::log::warn!(
                                            "Detected UI elements that use clipping, there are \
                                            some bugs under certain circumstances where the \
                                            clipping region is incorrect. You may want to \
                                            disable clipping if the UI element fails to \
                                            render correctly"
                                            );

                                            *has_shown_clipping_warning = true;
                                        }

                                        let scissor_region = ScissorRegion {
                                            x: x1 as u32,
                                            y: y1 as u32,
                                            width: width as u32,
                                            height: height as u32,
                                        };

                                        render_state = render_state.set_scissor(scissor_region);
                                        clip_stack.push(scissor_region);
                                    }
                                    Batch::ClipPop => {
                                        // Pop the last item off the clip stack and set the scissor
                                        // to the previous one
                                        clip_stack.pop();

                                        render_state =
                                            render_state.set_scissor(clip_stack.last().cloned());
                                    }
                                    Batch::None => (),
                                }
                            }

                            Ok(())
                        },
                    )
                },
            )
            .assume()
            .into_result()
            .expect("Could not render");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "v_pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "v_uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
    #[sem(name = "v_color", repr = "[f32; 4]", wrapper = "VertexColor")]
    Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct UiVert {
    pos: VertexPosition,
    uv: VertexUv,
    color: VertexColor,
}

#[derive(UniformInterface)]
struct UiUniformInterface {
    target_size: Uniform<[f32; 2]>,

    texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,

    /// Should be on eof the widget type constants below
    widget_type: Uniform<i32>,

    #[uniform(unbound)]
    text_box_transform: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    text_box_size: Uniform<[f32; 2]>,
}

/// Uniform widget type constant
const WIDGET_COLORED_TRIS: i32 = 0;
/// Uniform widget type constant
const WIDGET_IMAGE_TRIS: i32 = 1;
/// Uniform widget type constant
const WIDGET_TEXT: i32 = 2;

const PIXELATED_SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::Nearest,
    mag_filter: MagFilter::Nearest,
    depth_comparison: None,
};

// Quad vertices in a triangle fan
const QUAD_VERTS: [UiVert; 4] = [
    UiVert::new(
        VertexPosition::new([0.0, 0.0]),
        VertexUv::new([0.0, 0.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
    UiVert::new(
        VertexPosition::new([1.0, 0.0]),
        VertexUv::new([1.0, 0.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
    UiVert::new(
        VertexPosition::new([1.0, 1.0]),
        VertexUv::new([1.0, 1.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
    UiVert::new(
        VertexPosition::new([0.0, 1.0]),
        VertexUv::new([0.0, 1.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
];
