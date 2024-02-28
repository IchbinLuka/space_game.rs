use bevy::prelude::*;

use super::{fonts::FontsResource, TextHoverEffect};

#[inline]
pub fn default_hover_effect() -> TextHoverEffect {
    TextHoverEffect {
        normal_color: Color::WHITE,
        hover_color: Color::GRAY,
    }
}

#[inline]
pub fn default_font(res: &FontsResource) -> Handle<Font> {
    res.mouse_memoirs.clone()
}

#[inline]
pub fn text_button_style(res: &FontsResource,) -> TextStyle {
    TextStyle {
        font_size: 50.,
        color: Color::WHITE,
        font: default_font(res),
    }
}