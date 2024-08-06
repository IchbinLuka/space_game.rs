use bevy::prelude::*;

use super::{fonts::FontsResource, TextHoverEffect};

pub const SURFACE_COLOR: Color = Color::rgb(0., 0., 0.);
pub const SURFACE_COLOR_FOCUSED: Color = Color::rgb(0.1, 0.1, 0.1);

#[inline(always)]
pub fn default_hover_effect() -> TextHoverEffect {
    TextHoverEffect {
        normal_color: Color::WHITE,
        hover_color: Color::GRAY,
    }
}

#[inline(always)]
pub fn default_font(res: &FontsResource) -> Handle<Font> {
    res.mouse_memoirs.clone()
}

#[inline(always)]
pub fn text_button_style(res: &FontsResource) -> TextStyle {
    TextStyle {
        font_size: 50.,
        color: Color::WHITE,
        font: default_font(res),
    }
}

#[inline(always)]
pub fn text_button_small_style(res: &FontsResource) -> TextStyle {
    TextStyle {
        font_size: 40.,
        color: Color::WHITE,
        font: default_font(res),
    }
}

#[inline(always)]
pub fn text_title_style(res: &FontsResource) -> TextStyle {
    TextStyle {
        font_size: 100.,
        color: Color::WHITE,
        font: default_font(res),
    }
}

#[inline(always)]
pub fn text_title_style_small(res: &FontsResource) -> TextStyle {
    TextStyle {
        font_size: 60.,
        color: Color::WHITE,
        font: default_font(res),
    }
}

#[inline(always)]
pub fn text_body_style(res: &FontsResource) -> TextStyle {
    TextStyle {
        font_size: 30.,
        color: Color::WHITE,
        font: default_font(res),
    }
}

#[inline(always)]
pub fn fullscreen_center_style() -> Style {
    Style {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}
