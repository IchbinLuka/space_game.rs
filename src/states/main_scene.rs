use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;

use crate::model::settings::Settings;

use super::{DespawnOnCleanup, ON_GAME_STARTED};



fn scene_setup_3d(mut commands: Commands, settings: Res<Settings>) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });

    let mut transform = Transform::from_xyz(0.0, 40.0, 0.0);
    transform.rotate_x(-FRAC_PI_4);
    transform.rotate_y(FRAC_PI_4 * 0.7);

    commands.spawn((
        DespawnOnCleanup,
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                color: Color::hex("ffffff").unwrap(),
                shadows_enabled: settings.shadows_enabled,
                ..default()
            },
            transform,
            ..default()
        },
    ));
}


pub struct MainScenePlugin;
impl Plugin for MainScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ON_GAME_STARTED, scene_setup_3d);
    }
}