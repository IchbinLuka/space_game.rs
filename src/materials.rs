use bevy::app::{App, Plugin};

pub mod blink;
pub mod exhaust;
pub mod shield;
pub mod toon;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            toon::ToonMaterialPlugin,
            exhaust::ExhaustPlugin,
            shield::ShieldMaterialPlugin,
            blink::BlinkMaterialPlugin,
        ));
    }
}
