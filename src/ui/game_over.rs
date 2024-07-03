use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::{block_on, futures_lite::future, Task};
use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_round_ui::autosize::RoundUiAutosizeMaterial;
use serde::Deserialize;

use crate::api_constants::API_URL;
use crate::components::health::Health;
use crate::entities::space_station::SpaceStation;
use crate::states::{
    game_over, game_running, reset_physics_speed, slow_down_physics, AppState, DespawnOnCleanup,
};
use crate::ui::button::TextButtonBundle;
use crate::ui::fonts::FontsResource;
use crate::ui::theme::{fullscreen_center_style, text_button_style, text_title_style};

use super::game_hud::Score;
use super::theme::{text_body_style, text_title_style_small};
use super::UiRes;

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct BackToMenuButton;

#[derive(Deserialize, PartialEq)]
struct PlayerScore {
    score: u32,
    player_name: String,
    rank: u32,
}

#[derive(Component)]
enum Leaderboard {
    Loading(Task<Result<Vec<PlayerScore>, reqwest::Error>>),
    Loaded,
    Error,
}

async fn fetch_leaderboard(score: u32) -> Result<Vec<PlayerScore>, reqwest::Error> {
    let url = format!("{}/ranking_near_score/{}/3", API_URL, score);
    let response = reqwest::blocking::get(url)?;
    response.json::<Vec<PlayerScore>>()
}

fn poll_task_status(
    mut leader_boards: Query<(&mut Leaderboard, Entity)>,
    mut commands: Commands,
    font_res: Res<FontsResource>,
) {
    for (mut leaderboard, entity) in &mut leader_boards {
        let Leaderboard::Loading(ref mut task) = *leaderboard else {
            continue;
        };
        let Some(result) = block_on(future::poll_once(task)) else {
            continue;
        };

        commands.entity(entity).despawn_descendants();

        let Ok(scores) = result else {
            commands.entity(entity).with_children(|c| {
                c.spawn(TextBundle::from_section(
                    "Error loading leaderboard",
                    text_body_style(&font_res),
                ));
            });
            *leaderboard = Leaderboard::Error;
            continue;
        };

        commands.entity(entity).with_children(|c| {
            for score in scores {
                c.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        width: Val::Percent(100.),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|c| {
                    c.spawn(TextBundle::from_section(
                        format!("{}.", score.rank),
                        text_body_style(&font_res),
                    ));
                    c.spawn(TextBundle::from_section(
                        score.player_name,
                        text_body_style(&font_res),
                    ));
                    c.spawn(TextBundle::from_section(
                        score.score.to_string(),
                        text_body_style(&font_res),
                    ));
                });
            }
        });

        *leaderboard = Leaderboard::Loaded;
        // TODO: Remove the loading text and display the leaderboard
    }
}

fn game_over_events(
    mut game_over_events: EventReader<GameOverEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ in game_over_events.read() {
        next_state.set(AppState::GameOver);
    }
}

fn game_over_screen_setup(
    mut commands: Commands,
    font_res: Res<FontsResource>,
    mut rapier_config: ResMut<RapierConfiguration>,
    score: Res<Score>,
    ui_res: Res<UiRes>,
) {
    slow_down_physics(&mut rapier_config);
    commands
        .spawn((
            GameOverScreen,
            DespawnOnCleanup,
            NodeBundle {
                style: fullscreen_center_style(),
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn(TextBundle::from_section(
                t!("game_over"),
                text_title_style(&font_res),
            ));

            c.spawn(TextBundle {
                style: Style {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                ..TextBundle::from_section(
                    t!("score", score = score.value),
                    text_button_style(&font_res),
                )
            });
            let thread_pool = IoTaskPool::get();
            let score_value = score.value;
            let task: Task<Result<Vec<PlayerScore>, reqwest::Error>> =
                thread_pool.spawn(async move {
                    fetch_leaderboard(score_value).await
                    // Ok(vec![])
                });

            c.spawn((
                MaterialNodeBundle {
                    material: ui_res.card_background_material.clone(),
                    style: Style {
                        padding: UiRect::all(Val::Px(20.)),
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(400.),
                        ..default()
                    },
                    ..default()
                },
                RoundUiAutosizeMaterial,
            ))
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("leaderboard"),
                    text_title_style_small(&font_res),
                ));

                c.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(20.)),
                            ..default()
                        },
                        ..default()
                    },
                    Leaderboard::Loading(task),
                ))
                .with_children(|c| {
                    c.spawn(TextBundle::from_section(
                        t!("loading"),
                        text_body_style(&font_res),
                    ));
                });

                c.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        width: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|c| {
                    c.spawn((
                        TextButtonBundle::from_section(t!("restart"), text_button_style(&font_res)),
                        RestartButton,
                    ));

                    c.spawn((
                        TextButtonBundle::from_section(
                            t!("back_to_menu"),
                            text_button_style(&font_res),
                        ),
                        BackToMenuButton,
                    ));
                });
            });
        });
}

fn restart_game(
    restart_button: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    for interaction in &restart_button {
        if *interaction == Interaction::Pressed {
            reset_physics_speed(&mut rapier_config);
            next_state.set(AppState::MainSceneLoading);
        }
    }
}

fn back_to_menu(
    back_to_menu_button: Query<&Interaction, (Changed<Interaction>, With<BackToMenuButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    for interaction in &back_to_menu_button {
        if *interaction == Interaction::Pressed {
            reset_physics_speed(&mut rapier_config);
            next_state.set(AppState::StartScreenLoading);
        }
    }
}

#[allow(dead_code)]
fn trigger_game_over(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut space_stations: Query<&mut Health, With<SpaceStation>>,
) {
    if keyboard_input.pressed(KeyCode::KeyG) {
        for mut health in &mut space_stations {
            let max_health = health.max_health;
            health.take_damage(max_health);
        }
    }
}

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    game_over_events,
                    #[cfg(feature = "debug")]
                    trigger_game_over,
                )
                    .run_if(game_running()),
                (restart_game, back_to_menu, poll_task_status).run_if(game_over()),
            ),
        )
        .add_systems(OnEnter(AppState::GameOver), game_over_screen_setup)
        .add_event::<GameOverEvent>();
    }
}
