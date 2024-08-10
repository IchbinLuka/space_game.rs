use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::{block_on, futures_lite::future, Task};
use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_round_ui::autosize::RoundUiAutosizeMaterial;
use bevy_simple_text_input::{TextInputBundle, TextInputInactive, TextInputValue};
use space_game_common::ScoreEvent;

use crate::components::health::Health;
use crate::entities::space_station::SpaceStation;
use crate::model::settings::Settings;
use crate::states::{
    game_over, game_running, reset_physics_speed, slow_down_physics, AppState, DespawnOnCleanup,
};
use crate::ui::fonts::FontsResource;
use crate::ui::theme::{fullscreen_center_style, text_button_style, text_title_style};
use crate::ui::widgets::TextButtonBundle;
use crate::utils::api::{ApiManager, Token};

use super::game_hud::Score;
use super::leaderboard::{AddLeaderboardExtension, FetchLeaderboardRequest};
use super::theme::text_title_style_small;
use super::widgets::FocusTextInputOnInteraction;
use super::UiRes;

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct BackToMenuButton;

#[derive(Component)]
struct SubmitButton {
    text_field: Entity,
}

fn game_over_events(
    mut game_over_events: EventReader<GameOverEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ in game_over_events.read() {
        next_state.set(AppState::GameOver);
    }
}

fn submit_score_if_logged_in(mut commands: Commands, score: Res<Score>, settings: Res<Settings>) {
    let Some(token) = settings.api_token.as_ref() else {
        return;
    };

    commands.add(SubmitScore {
        events: score.events.clone(),
        token: token.clone(),
    });
}

fn game_over_screen_setup(
    mut commands: Commands,
    font_res: Res<FontsResource>,
    mut rapier_config: ResMut<RapierConfiguration>,
    score: Res<Score>,
    ui_res: Res<UiRes>,
    api_manager: Res<ApiManager>,
    settings: Res<Settings>,
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

            if settings.api_token.is_none() {
                c.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|c| {
                    let text_field = c
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(200.),
                                    height: Val::Px(50.),
                                    ..default()
                                },
                                ..default()
                            },
                            FocusTextInputOnInteraction,
                            TextInputBundle::default()
                                .with_text_style(TextStyle {
                                    font_size: 40.,
                                    color: Color::WHITE,
                                    ..default()
                                })
                                .with_text_style(text_button_style(&font_res))
                                .with_placeholder(t!("enter_name"), None)
                                .with_inactive(true),
                        ))
                        .id();

                    c.spawn((
                        TextButtonBundle::from_section(t!("submit"), text_button_style(&font_res)),
                        SubmitButton { text_field },
                    ));
                });
            }

            let score_value = score.value;

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

                c.add_leaderboard(
                    match &settings.api_token {
                        Some(token) => FetchLeaderboardRequest::NearPlayer {
                            token: token.clone(),
                        },
                        None => FetchLeaderboardRequest::NearScore { score: score_value },
                    },
                    3,
                    api_manager.clone(),
                    &font_res,
                );

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

#[derive(Component)]
pub struct CreatePlayerTask(Task<Result<Token, reqwest::Error>>);

#[derive(Component)]
pub struct SubmitScoreTask(Task<Result<(), reqwest::Error>>);

fn submit_score(
    submit_button: Query<(&Interaction, &SubmitButton), Changed<Interaction>>,
    mut text_fields: Query<(&TextInputValue, &mut TextInputInactive)>,
    mut commands: Commands,
    api_manager: Res<ApiManager>,
) {
    for (interaction, button) in &submit_button {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((text_field_value, mut inactive)) = text_fields.get_mut(button.text_field) else {
            error!("No text field found for submit button");
            continue;
        };

        let trimmed = text_field_value.0.trim();

        if trimmed.is_empty() || trimmed.len() > 15 {
            error!("Player name is empty");
            // TODO: show error message
            continue;
        }

        inactive.0 = true;

        let player_name = trimmed.to_string();
        let task_pool = IoTaskPool::get();
        let api_manager = api_manager.clone();
        let task = task_pool.spawn(async move {
            async_compat::Compat::new(api_manager.create_player(player_name)).await
        });
        commands.spawn(CreatePlayerTask(task));
    }
}

fn poll_submit_score(
    mut commands: Commands,
    mut submit_score_tasks: Query<(Entity, &mut SubmitScoreTask)>,
) {
    for (entity, mut task) in &mut submit_score_tasks {
        let Some(result) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        commands.entity(entity).despawn();

        match result {
            Ok(_) => {
                info!("Score submitted successfully");
            }
            Err(e) => {
                error!("Error submitting score: {:?}", e);
            }
        }
    }
}

fn poll_player_creation(
    mut commands: Commands,
    mut create_player_tasks: Query<(Entity, &mut CreatePlayerTask)>,
    mut settings: ResMut<Settings>,
    score: Res<Score>,
) {
    for (entity, mut task) in &mut create_player_tasks {
        let Some(result) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        commands.entity(entity).despawn();

        let Ok(token) = result else {
            error!("Error creating player: {:?}", result);
            continue;
        };

        settings.api_token = Some(token.clone());

        // Now that we have a token, we can submit the score
        commands.add(SubmitScore {
            events: score.events.clone(),
            token,
        });
    }
}

struct SubmitScore {
    events: Vec<ScoreEvent>,
    token: Token,
}

impl Command for SubmitScore {
    fn apply(self, world: &mut World) {
        let api_manager = world
            .get_resource::<ApiManager>()
            .expect("ApiManager missing")
            .clone();

        let task_pool = IoTaskPool::get();
        let task = task_pool.spawn(async move {
            async_compat::Compat::new(api_manager.submit_score(&self.events, &self.token)).await
        });

        world.spawn(SubmitScoreTask(task));
    }
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
        app.insert_resource(ApiManager::new())
            .add_systems(
                Update,
                (
                    (
                        game_over_events,
                        #[cfg(feature = "debug")]
                        trigger_game_over,
                    )
                        .run_if(game_running()),
                    (
                        restart_game,
                        back_to_menu,
                        submit_score,
                        poll_player_creation,
                        poll_submit_score,
                    )
                        .run_if(game_over()),
                ),
            )
            .add_systems(
                OnEnter(AppState::GameOver),
                (game_over_screen_setup, submit_score_if_logged_in),
            )
            .add_event::<GameOverEvent>();
    }
}
