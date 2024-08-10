use bevy::app::Plugin;

pub mod api;
pub mod asset_loading;
pub mod clipboard;
pub mod collisions;
pub mod materials;
pub mod math;
pub mod misc;
pub mod scene;
pub mod sets;
pub mod tasks;

pub struct UtilsPlugin;
impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<clipboard::Clipboard>();
    }
}
