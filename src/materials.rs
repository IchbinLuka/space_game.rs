use bevy::app::{App, Plugin};

pub mod outline;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((outline::OutlineMaterialPlugin,));
    }
}
