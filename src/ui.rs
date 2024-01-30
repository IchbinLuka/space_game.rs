use bevy::app::{App, Plugin};

pub mod enemy_indicator;
pub mod fonts;
pub mod health_bar;
pub mod health_bar_3d;
pub mod score;
pub mod sprite_3d_renderer;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            health_bar::HealthBarPlugin,
            sprite_3d_renderer::Sprite3DRendererPlugin,
            score::ScorePlugin,
            fonts::FontsPlugin,
            enemy_indicator::EnemyIndicatorPlugin,
            health_bar_3d::HealthBar3DPlugin,
        ));
    }
}
