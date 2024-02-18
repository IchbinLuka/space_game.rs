use bevy::prelude::*;
use bevy_round_ui::{
    autosize::RoundUiAutosizeMaterial,
    prelude::{RoundUiBorder, RoundUiMaterial},
};

use crate::{components::health::Health, entities::spaceship::IsPlayer, states::{game_running, ON_GAME_STARTED}};


#[derive(Component)]
struct HealthBarContent;

fn health_bar_setup(mut commands: Commands, mut materials: ResMut<Assets<RoundUiMaterial>>) {
    const PANEL_WIDTH: f32 = 400.;
    const PANEL_HEIGHT: f32 = 40.;
    const PADDING: f32 = 5.;

    commands
        .spawn(MaterialNodeBundle {
            style: Style {
                left: Val::Px(10.),
                bottom: Val::Px(10.),
                position_type: PositionType::Absolute,
                width: Val::Px(PANEL_WIDTH),
                height: Val::Px(PANEL_HEIGHT),
                padding: UiRect::all(Val::Px(PADDING)),
                ..default()
            },
            material: materials.add(RoundUiMaterial {
                background_color: Color::BLACK,
                border_radius: RoundUiBorder::all(PANEL_HEIGHT / 2.).into(),
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
                        border_radius: RoundUiBorder::all((PANEL_HEIGHT - PADDING * 2.) / 2.)
                            .into(),
                        ..default()
                    }),
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
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
    player_query: Query<&Health, (IsPlayer, Changed<Health>)>,
    mut health_bar_query: Query<&mut Style, With<HealthBarContent>>,
) {
    let Ok(player_health) = player_query.get_single() else {
        return;
    };
    for mut style in &mut health_bar_query {
        style.width = Val::Percent(player_health.health / player_health.max_health * 100.);
    }
}

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ON_GAME_STARTED, health_bar_setup)
            .add_systems(
                Update,
                health_bar_update.run_if(game_running()),
            );
    }
}
