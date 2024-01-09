use bevy::prelude::*;
use bevy_round_ui::{
    autosize::RoundUiAutosizeMaterial,
    prelude::{RoundUiBorder, RoundUiMaterial},
};

use crate::{
    entities::spaceship::{player::Player, Health},
    AppState,
};

#[derive(Component)]
struct HealthBarContent;

fn health_bar_setup(mut commands: Commands, mut materials: ResMut<Assets<RoundUiMaterial>>) {
    const PANEL_WIDTH: f32 = 400.0;
    const PANEL_HEIGHT: f32 = 40.0;
    const PADDING: f32 = 5.0;

    commands
        .spawn(MaterialNodeBundle {
            style: Style {
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(PANEL_WIDTH),
                height: Val::Px(PANEL_HEIGHT),
                padding: UiRect::all(Val::Px(PADDING)),
                ..default()
            },
            material: materials.add(RoundUiMaterial {
                background_color: Color::BLACK,
                border_radius: RoundUiBorder::all(PANEL_HEIGHT / 2.0).into(),
                size: Vec2::new(PANEL_WIDTH, PANEL_HEIGHT),
                ..default()
            }),
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                MaterialNodeBundle {
                    material: materials.add(RoundUiMaterial {
                        background_color: Color::hex("#ef4d34").unwrap(),
                        border_radius: RoundUiBorder::all((PANEL_HEIGHT - PADDING * 2.0) / 2.0)
                            .into(),
                        ..default()
                    }),
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                RoundUiAutosizeMaterial,
                HealthBarContent,
            ));
        });
}

fn health_bar_update(
    player_query: Query<&Health, With<Player>>,
    mut health_bar_query: Query<&mut Style, With<HealthBarContent>>,
) {
    let Ok(player_health) = player_query.get_single() else {
        return;
    };
    for mut style in &mut health_bar_query {
        style.width = Val::Percent(player_health.0);
    }
}

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Running), health_bar_setup)
            .add_systems(
                Update,
                health_bar_update.run_if(in_state(AppState::Running)),
            );
    }
}
