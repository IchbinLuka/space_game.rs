use bevy::app::{Plugin, App};

pub mod health_bar;


pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            health_bar::HealthBarPlugin, 
        ));
    }
}