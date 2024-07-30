use bevy::prelude::*;
use bevy_round_ui::prelude::RoundUiMaterial;

pub mod fonts;
pub mod game_hud;
pub mod game_over;
pub mod health_bar_3d;
pub mod leaderboard;
pub mod minimap;
pub mod settings;
pub mod sprite_3d_renderer;
pub mod theme;
pub mod widgets;

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

fn hover_effect_cursor(
    query: Query<&Interaction, Changed<Interaction>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    for interaction in &query {
        for mut window in &mut windows {
            window.cursor.icon = if *interaction == Interaction::Hovered {
                CursorIcon::Pointer
            } else {
                CursorIcon::Default
            };
        }
    }
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

fn ui_setup(mut commands: Commands, mut materials: ResMut<Assets<RoundUiMaterial>>) {
    commands.insert_resource(UiRes {
        card_background_material: materials.add(RoundUiMaterial {
            background_color: Color::rgb(0., 0., 0.),
            border_radius: Vec4::splat(30.),
            ..default()
        }),
    })
}

#[derive(Resource)]
pub struct UiRes {
    pub card_background_material: Handle<RoundUiMaterial>,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ui_setup)
            .add_systems(
                Update,
                (hover_effect_node, hover_effect_text, hover_effect_cursor),
            )
            .add_plugins((
                game_hud::GameHudPlugin,
                sprite_3d_renderer::Sprite3DRendererPlugin,
                fonts::FontsPlugin,
                health_bar_3d::HealthBar3DPlugin,
                settings::SettingsPlugin,
                widgets::WidgetsPlugin,
                minimap::MinimapPlugin,
                game_over::GameOverPlugin,
                leaderboard::LeaderboardPlugin,
            ));
    }
}
