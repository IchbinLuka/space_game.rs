use bevy::app::{App, Plugin};

pub mod health_bar;
pub mod node_3d_renderer;
pub mod score;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            health_bar::HealthBarPlugin, 
            node_3d_renderer::Node3DRendererPlugin,
            score::ScorePlugin,
        ));
    }
}
