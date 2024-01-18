use bevy::app::{App, Plugin};

pub mod health_bar;
pub mod sprite_3d_renderer;
pub mod score;
pub mod fonts;
pub mod enemy_indicator;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            health_bar::HealthBarPlugin, 
            sprite_3d_renderer::Sprite3DRendererPlugin,
            score::ScorePlugin,
            fonts::FontsPlugin,
            enemy_indicator::EnemyIndicatorPlugin,
        ));
    }
}
