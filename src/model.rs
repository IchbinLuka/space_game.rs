use bevy::app::Plugin;

pub mod settings;

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((settings::SettingsPlugin,));
    }
}
