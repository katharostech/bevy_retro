//! Bevy Retrograde physics plugin

use bevy::render2::texture::Image;
use bevy::{ecs::component::ComponentDescriptor, prelude::*};
#[cfg(feature = "debug")]
use bevy_retrograde_core::prelude::AppBuilderRenderHookExt;
use density_mesh_core::prelude::GenerateDensityMeshSettings;
use density_mesh_core::prelude::PointsSeparation;

pub use heron;
pub use heron::prelude::*;

#[doc(hidden)]
pub mod prelude {
    pub use crate::RetroPhysicsPlugin;
}

#[cfg(feature = "debug")]
mod render_hook;
use image::ImageBuffer;
#[cfg(feature = "debug")]
use render_hook::PhysicsDebugRenderHook;

/// Physics plugin for Bevy Retrograde
pub struct RetroPhysicsPlugin;

#[cfg(feature = "debug")]
use bevy_retrograde_core::prelude::Color;
#[cfg(feature = "debug")]
#[derive(Clone, Debug)]
pub enum PhysicsDebugRendering {
    Disabled,
    Enabled { color: Color },
}

#[cfg(feature = "debug")]
impl Default for PhysicsDebugRendering {
    fn default() -> Self {
        Self::Disabled
    }
}

impl Plugin for RetroPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PhysicsPlugin::default());

        #[cfg(feature = "debug")]
        app.add_render_hook::<PhysicsDebugRenderHook>()
            .init_resource::<PhysicsDebugRendering>();

        app.register_component(ComponentDescriptor::new::<TesselatedColliderHasLoaded>(
            bevy::ecs::component::StorageType::SparseSet,
        ))
        .add_system_to_stage(CoreStage::PostUpdate, generate_colliders);
    }
}

/// Create a convex hull [`CollisionShape`] from a sprite image based on it's alpha channel
///
/// Returns [`None`] if a mesh for the given image could not be generated
pub fn create_convex_collider(
    image: DynamicImage,
    tesselator_config: &TesselatedColliderConfig,
) -> Option<CollisionShape> {
    use density_mesh_core::prelude::DensityMeshGenerator;
    use density_mesh_image::settings::GenerateDensityImageSettings;
    let width = image.width();
    let height = image.height();
    let density_map = density_mesh_image::generate_densitymap_from_image(
        image,
        &GenerateDensityImageSettings {
            density_source: density_mesh_image::settings::ImageDensitySource::Alpha,
            scale: 1,
        },
    )
    .ok()?;

    let mut density_mesh_generator = DensityMeshGenerator::new(
        vec![],
        density_map,
        GenerateDensityMeshSettings {
            extrude_size: if tesselator_config.extrusion != 0.0 {
                Some(tesselator_config.extrusion)
            } else {
                None
            },
            points_separation: PointsSeparation::Constant(tesselator_config.vertice_separation),
            ..Default::default()
        },
    );

    density_mesh_generator.process_wait().ok()?;

    let density_mesh = density_mesh_generator.into_mesh()?;

    let points = density_mesh
        .points
        .iter()
        .map(|point| {
            Vec3::new(
                (point.x - width as f32 / 2.0) + 0.5,
                -(point.y - height as f32 / 2.0) - 0.5,
                0.,
            )
        })
        .collect::<Vec<_>>();

    Some(CollisionShape::ConvexHull {
        points,
        border_radius: if tesselator_config.vertice_radius != 0.0 {
            Some(tesselator_config.vertice_radius)
        } else {
            None
        },
    })
}

struct TesselatedColliderHasLoaded;

use image::DynamicImage;
use image::GenericImageView;

/// Sprite collision tesselator config
#[derive(Debug, Clone)]
pub struct TesselatedColliderConfig {
    /// The minimum separation between generated vertices. This is, in effect, controls the
    /// "resolution" of the mesh, with a value of 0 meaning that vertices may be placed on each
    /// individual pixel, producing the maximum accuracy convex collision shape.
    ///
    /// **Default:** `0.0`
    pub vertice_separation: f32,
    /// The distance to extrude the generated mesh. Adding an extrusion can prevent panics from
    /// being caused when you try to tesselate a collision shape that is only one pixel high.
    ///
    /// When a collision shape is only one pixel high, only two vertices will be created, which is a
    /// mesh with no interior and therefore no convex "shape". This causes panics when such a shape
    /// comes in contact with another one.
    ///
    /// Adding a small extrusion will make sure that even a 2 vertice mesh will get extruded to a 4
    /// vertice mesh that has an interior and will collide properly.
    ///
    /// **Default:** `0.1`
    pub extrusion: f32,
    /// Vertices will be generated in the center of pixels. This means that, without any vertice
    /// radius and no extrusion, the sprite collision will overlap with objects in comes in contact
    /// with by half a pixel. By setting the vertice radius to 0.5, an extra half-pixel buffer will
    /// be added making the collision appear as expected. This can be tweaked in combination with
    /// the extrusion value to control the buffer around the generated sprite mesh.
    ///
    /// **Default:** `0.4`
    pub vertice_radius: f32,
}

impl Default for TesselatedColliderConfig {
    fn default() -> Self {
        Self {
            vertice_separation: 10.,
            extrusion: 0.1,
            vertice_radius: 0.4,
        }
    }
}

/// A component used to automatically add a [`CollisionShape`] to an entity that is generated
/// automatically by tesselating [`Image`] collision shape based on it's alpha channel
#[derive(Default)]
pub struct TesselatedCollider {
    pub texture: Handle<Image>,
    pub tesselator_config: TesselatedColliderConfig,
}

fn generate_colliders(
    mut commands: Commands,
    pending_colliders: Query<(Entity, &TesselatedCollider), Without<TesselatedColliderHasLoaded>>,
    image_assets: Res<Assets<Image>>,
) {
    // TODO: Hot reload collision shape changes
    for (ent, tesselated_collider) in pending_colliders.iter() {
        // Get the collider image
        let image = if let Some(image) = image_assets.get(&tesselated_collider.texture) {
            image
        } else {
            continue;
        };

        let shape = create_convex_collider(
            DynamicImage::ImageRgba8(
                ImageBuffer::from_vec(
                    image.texture_descriptor.size.width,
                    image.texture_descriptor.size.height,
                    image.data.clone(),
                )
                .unwrap(),
            ),
            &tesselated_collider.tesselator_config,
        )
        .expect("Could not generate collision shape from image");

        commands
            .entity(ent)
            .insert(shape)
            .insert(TesselatedColliderHasLoaded);
    }
}
