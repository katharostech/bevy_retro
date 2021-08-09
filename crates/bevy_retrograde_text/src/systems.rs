use bdf::Glyph;
use bevy::render2::{
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::Image,
};
use image::{GenericImage, Rgba, RgbaImage};
use unicode_linebreak::BreakOpportunity;

use crate::*;

trait GlyphExt {
    fn real_width(&self) -> u32;
}

pub(crate) fn font_rendering(
    mut texts: Query<
        (
            Entity,
            &Text,
            &Handle<Font>,
            Option<&TextBlock>,
            Option<&mut Handle<Image>>,
        ),
        Or<(
            Added<Text>,
            Added<Handle<Font>>,
            Added<TextBlock>,
            Changed<Text>,
            Changed<Handle<Font>>,
            Changed<TextBlock>,
            With<TextNeedsUpdate>,
        )>,
    >,
    mut commands: Commands,
    font_assets: Res<Assets<Font>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    // For all update text entities
    for (ent, text, font_handle, text_block, image_handle) in texts.iter_mut() {
        // The block below fixes inferrence in Rust Analyzer 🤷‍♂️. It shouldn't be necessary once that's fixed
        let text: &Text = text;
        let text_block: Option<&TextBlock> = text_block;
        let image_handle: Option<Mut<Handle<Image>>> = image_handle;

        // Try to load the font
        let font = if let Some(font) = font_assets.get(font_handle) {
            font
        } else {
            // Mark this text as needing an update if the font has not been loading yet so we can
            // come back to it later
            commands.entity(ent).insert(TextNeedsUpdate);
            continue;
        };

        // Remove text update flag now that we are updating it
        commands.entity(ent).remove::<TextNeedsUpdate>();

        let image = rasterize_text_block(text, font, text_block);

        // Update or add the new image handle to the entity
        let new_image_handle = image_assets.add(Image::new(
            Extent3d {
                width: image.width(),
                height: image.height(),
                ..Default::default()
            },
            TextureDimension::D2,
            image.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
        ));
        if let Some(mut handle) = image_handle {
            image_assets.remove(&*handle);
            *handle = new_image_handle;
        } else {
            commands.entity(ent).insert(new_image_handle);
        }
    }
}

/// Get the image for a text block
///
/// This function should not be necessary for normal users, but can be useful in advanced situations
/// when you whish to rasterize a text block manually.
pub fn rasterize_text_block(
    text: &Text,
    font: &Font,
    text_block: Option<&TextBlock>,
) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
    let default_glyph = font.glyphs.get(&' ');
    let font_bounds = &font.bounds;

    // Calculate line breaks for the text
    let mut line_breaks = unicode_linebreak::linebreaks(&text.text).collect::<Vec<_>>();
    line_breaks.reverse();
    let line_breaks = line_breaks; // Make immutable

    // Create a vector that holds all of the lines of the text and the glyphs in each line
    let mut lines: Vec<Vec<Glyph>> = Default::default();

    // The height of a line
    let line_height = font.bounds.height;

    // Start glyph layout
    let mut current_line = Vec::new();
    let mut line_x = 0; // The x position in the line we are currently at
    for (char_i, char) in text.text.char_indices() {
        // Get the glyph for this character
        let glyph = font
            .glyphs
            .get(&char)
            .or(default_glyph)
            .unwrap_or_else(|| panic!("Font does not contain glyph for character: {:?}", char));

        // Add the next glyph to the current line
        current_line.push(glyph.clone());

        // Wrap the line if necessary
        if let Some(max_width) = text_block.map(|x| x.width) {
            // Calculate the new x position of the line after adding this glyph
            line_x += glyph.device_width.0;

            // If this character must break the line
            if line_breaks
                .iter()
                .any(|(i, op)| i == &(char_i + 1) && op == &BreakOpportunity::Mandatory)
                // The last character always breaks, but we want to ignore that one
                && char_i != text.text.len() - 1
            {
                // Add this line to the lines list
                lines.push(current_line);
                // Start a new line
                current_line = Vec::new();
                // Reset the line x position
                line_x = 0;

            // If the new line x goes over our max width, we need to find the last position we
            // can break the line
            } else if line_x > max_width {
                for (break_i, line_break) in &line_breaks {
                    match (break_i, line_break) {
                        // We found a spot that we can break the line
                        (split_i, unicode_linebreak::BreakOpportunity::Allowed)
                            if split_i < &char_i =>
                        {
                            // Figure out how many character will be broken off
                            let broken_chars = char_i - split_i;
                            // Get the point in the line at which to break it
                            let split_at = current_line.len() - 1 - broken_chars;
                            // Split the broken off characters into a new line
                            let next_line = current_line.split_off(split_at);
                            // Add the current line to the lines list
                            lines.push(current_line);
                            // Set the new current line to the next line
                            current_line = next_line;
                            // Reset our current line x counter to the length of the new current
                            // line
                            line_x = current_line
                                .iter()
                                .fold(0, |width, g| width + g.device_width.0);
                            break;
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    lines.push(current_line);

    // Get the height of the lines of the text block
    let lines_height = line_height * lines.len() as u32;

    // Calculate the height and width of the text block image
    let image_height = lines_height.max(text_block.map(|x| x.height).flatten().unwrap_or(0));
    let image_width = lines.iter().fold(0, |width, line| {
        let line_width = line
            .iter()
            .fold(0, |width, glyph| width + glyph.device_width.0);

        if line_width > width {
            line_width
        } else {
            width
        }
    }) as u32;
    // Make sure image is at least as wide as the specified text block width
    let image_width = text_block
        .map(|x| x.width.max(image_width))
        .unwrap_or(image_width);

    // Create a new image the size of the text box
    let mut image = RgbaImage::new(image_width, image_height);

    // Calculate the y offset to account for vertical alignment
    let y_offset = text_block
        .map(|block| match (block.height, &block.vertical_align) {
            (None, _) => 0,
            (_, TextVerticalAlign::Top) => 0,
            (Some(_), TextVerticalAlign::Middle) => (image_height - lines_height) / 2,
            (Some(_), TextVerticalAlign::Bottom) => image_height - lines_height,
        })
        .unwrap_or(0);

    // Loop through all the lines
    for (line_i, line) in lines.iter().enumerate() {
        let line_y = line_i as u32 * line_height;
        let mut line_x = 0u32;

        // Calculate the x offset to account for text alignment
        let x_offset = text_block
            .map(|block| match &block.horizontal_align {
                TextHorizontalAlign::Left => 0,
                other => {
                    // Get the full width of the characters in this line
                    let chars_width = line
                        .iter()
                        .fold(0, |width, glyph| width + glyph.device_width.0);

                    match other {
                        TextHorizontalAlign::Center => {
                            (image_width - chars_width.min(image_width)) / 2
                        }
                        TextHorizontalAlign::Right => image_width - chars_width.min(image_width),
                        _ => 0, // unreachable, but this works, too
                    }
                }
            })
            .unwrap_or(0);

        // Loop through all the glyphs in each line
        for glyph in line {
            // Get bounds
            let bounds = &glyph.bounds;

            // Skip rasterizing whitespace chars
            if !glyph.codepoint.is_whitespace() {
                // Create a sub-image of the text block for the area occupied by the glyph
                let mut sub_img = image.sub_image(
                    line_x + x_offset,
                    line_y + y_offset,
                    bounds.width,
                    bounds.height,
                );

                for x in 0..bounds.width {
                    for y in 0..bounds.height {
                        let pixel = sub_img.get_pixel_mut(
                            x,
                            (y as i32 + font_bounds.height as i32 + font_bounds.y
                                - bounds.height as i32
                                - bounds.y) as u32,
                        );

                        *pixel = Rgba([
                            (255. * text.color.r()).round() as u8,
                            (255. * text.color.g()).round() as u8,
                            (255. * text.color.b()).round() as u8,
                            if glyph.bitmap.get(x, y) {
                                (255. * text.color.a()).round() as u8
                            } else {
                                0
                            },
                        ]);
                    }
                }
            }

            // Increment line position
            line_x += glyph.device_width.0;
        }
    }

    image
}
