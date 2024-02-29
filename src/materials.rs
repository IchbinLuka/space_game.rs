use bevy::app::{App, Plugin};

pub mod toon;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((toon::ToonMaterialPlugin,));
    }
}