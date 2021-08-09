use bevy::{prelude::*, sprite2::Sprite};

use crate::prelude::*;

/// Marker component indicating that a text entity needs to be updated but hasn't yet because it's
/// assets are not loaded.
pub(crate) struct TextNeedsUpdate;

#[derive(Bundle, Default, Debug, Clone)]
pub struct TextBundle {
    pub font: Handle<Font>,
    pub text: Text,
    pub sprite: Sprite,
    pub visible: Visible,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// The text inside a text entity or text block
#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: Color::WHITE,
        }
    }
}

/// The configuration for a text block
#[derive(Debug, Clone)]
pub struct TextBlock {
    pub width: u32,
    pub horizontal_align: TextHorizontalAlign,
    pub height: Option<u32>,
    pub vertical_align: TextVerticalAlign,
}

impl Default for TextBlock {
    fn default() -> Self {
        TextBlock {
            width: 100,
            horizontal_align: TextHorizontalAlign::Left,
            height: None,
            vertical_align: TextVerticalAlign::Top,
        }
    }
}

/// The alignment of text horizontally
#[derive(Debug, Clone)]
pub enum TextHorizontalAlign {
    Left,
    Center,
    Right,
}

/// The alignment of text vertically
#[derive(Debug, Clone)]
pub enum TextVerticalAlign {
    Top,
    Middle,
    Bottom,
}
