use bevy::{
    app::{App, Plugin, Update},
    ecs::{component::Component, query::Changed, system::Query},
    render::color::Color,
    text::Text,
    ui::{BackgroundColor, Interaction},
};

pub mod button;
pub mod fonts;
pub mod game_hud;
pub mod game_over;
pub mod health_bar_3d;
pub mod minimap;
pub mod settings;
pub mod sprite_3d_renderer;
pub mod theme;

#[derive(Component, Default)]
pub struct NodeHoverEffect {
    pub normal_color: Color,
    pub hover_color: Color,
}

#[derive(Component, Default)]
pub struct TextHoverEffect {
    pub normal_color: Color,
    pub hover_color: Color,
}

fn hover_effect_node(
    mut query: Query<(&NodeHoverEffect, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (hover_effect, interaction, mut background_color) in &mut query {
        match *interaction {
            Interaction::None => {
                background_color.0 = hover_effect.normal_color;
            }
            Interaction::Hovered => {
                background_color.0 = hover_effect.hover_color;
            }
            _ => {}
        }
    }
}

fn hover_effect_text(
    mut query: Query<(&TextHoverEffect, &Interaction, &mut Text), Changed<Interaction>>,
) {
    for (hover_effect, interaction, mut text) in &mut query {
        let color = match *interaction {
            Interaction::None => hover_effect.normal_color,
            Interaction::Hovered => hover_effect.hover_color,
            _ => {
                continue;
            }
        };

        for section in text.sections.iter_mut() {
            section.style.color = color;
        }
    }
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (hover_effect_node, hover_effect_text))
            .add_plugins((
                game_hud::GameHudPlugin,
                sprite_3d_renderer::Sprite3DRendererPlugin,
                fonts::FontsPlugin,
                health_bar_3d::HealthBar3DPlugin,
                settings::SettingsPlugin,
                button::ButtonPlugin,
                minimap::MinimapPlugin,
                game_over::GameOverPlugin,
            ));
    }
}
