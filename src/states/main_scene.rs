use std::{f32::consts::FRAC_PI_4, time::Duration};

use bevy::{prelude::*, time::Stopwatch};

use crate::{
    components::despawn_after::DespawnTimer,
    model::settings::Settings,
    ui::{fonts::FontsResource, theme::text_body_style},
};

use super::{game_running, DespawnOnCleanup, ON_GAME_STARTED};

#[derive(Resource, Deref, DerefMut)]
pub struct GameTime(pub Stopwatch);
impl GameTime {
    fn new() -> Self {
        Self(Stopwatch::new())
    }
}

fn game_time(mut game_time: ResMut<GameTime>, time: Res<Time>) {
    game_time.tick(time.delta());
}

fn main_scene_setup(mut commands: Commands, settings: Res<Settings>, font_res: Res<FontsResource>) {
    commands.insert_resource(GameTime::new());

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
                color: Color::WHITE,
                shadows_enabled: settings.shadows_enabled,
                ..default()
            },
            transform,
            ..default()
        },
    ));

    commands
        .spawn((
            DespawnOnCleanup,
            DespawnTimer::new(Duration::from_secs(5)),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn(TextBundle {
                style: Style {
                    top: Val::Percent(30.),
                    ..default()
                },
                ..TextBundle::from_section(
                    t!("protect_space_stations"),
                    TextStyle {
                        font_size: 50.,
                        ..text_body_style(&font_res)
                    },
                )
            });
        });
}

pub struct MainScenePlugin;
impl Plugin for MainScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, game_time.run_if(game_running()))
            .add_systems(ON_GAME_STARTED, main_scene_setup);
    }
}
