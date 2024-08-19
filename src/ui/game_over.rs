use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_simple_text_input::{TextInputBundle, TextInputInactive, TextInputValue};

use crate::components::health::Health;
use crate::entities::space_station::SpaceStation;
use crate::model::settings::{Profile, Settings};
use crate::states::{
    game_over, game_running, reset_physics_speed, slow_down_physics, AppState, DespawnOnCleanup,
};
use crate::ui::fonts::FontsResource;
use crate::ui::theme::{fullscreen_center_style, text_button_style, text_title_style};
use crate::ui::widgets::TextButtonBundle;
use crate::utils::api::ApiManager;
use crate::utils::tasks::{poll_task, StartJob};

use super::game_hud::Score;
use super::leaderboard::{AddLeaderboardExtension, FetchLeaderboardRequest};
use super::theme::text_title_style_small;
use super::ui_card;
use super::widgets::FocusTextInputOnInteraction;

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

fn submit_score_if_logged_in(
    mut commands: Commands,
    score: Res<Score>,
    settings: Res<Settings>,
    api_manager: Res<ApiManager>,
) {
    let Some(profile) = settings.profile.as_ref() else {
        return;
    };

    let api_manager = api_manager.clone();
    let profile = profile.clone();
    let events = score.events.clone();

    commands.add(StartJob {
        job: Box::pin(async move {
            api_manager
                .submit_score(&events, &profile.token.clone())
                .await
        }),
        on_complete: |result, _| match result {
            Ok(_) => {
                info!("Score submitted successfully");
            }
            Err(e) => {
                error!("Failed to submit score: {:?}", e);
            }
        },
    });
}

fn game_over_screen_setup(
    mut commands: Commands,
    font_res: Res<FontsResource>,
    mut rapier_config: ResMut<RapierConfiguration>,
    score: Res<Score>,
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

            if settings.profile.is_none() {
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

            c.spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(20.)),
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(400.),
                        ..default()
                    },
                    ..ui_card()
                })
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("leaderboard"),
                    text_title_style_small(&font_res),
                ));

                c.add_leaderboard(
                    match &settings.profile {
                        Some(profile) => FetchLeaderboardRequest::NearPlayer {
                            token: profile.token.clone(),
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

#[derive(Debug)]
enum SubmitScoreError {
    PlayerCreationFailed,
    ScoreSubmissionFailed,
}

fn submit_score(
    submit_button: Query<(&Interaction, &SubmitButton, Entity), Changed<Interaction>>,
    mut text_fields: Query<(&TextInputValue, &mut TextInputInactive, Entity)>,
    mut commands: Commands,
    api_manager: Res<ApiManager>,
    score: Res<Score>,
) {
    for (interaction, button, entity) in &submit_button {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((text_field_value, mut inactive, text_field)) =
            text_fields.get_mut(button.text_field)
        else {
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
        let api_manager = api_manager.clone();

        let score_events = score.events.clone();

        commands.add(StartJob {
            job: Box::pin(async move {
                let Ok(profile) = api_manager.create_player(player_name).await else {
                    error!("Error creating player");
                    return Err(SubmitScoreError::PlayerCreationFailed);
                };
                api_manager
                    .submit_score(&score_events, &profile.token)
                    .await
                    .map_err(|_| SubmitScoreError::ScoreSubmissionFailed)?;

                Ok(profile)
            }),
            on_complete: |result, world: &mut World| {
                let Ok(profile) = result else {
                    error!("Could not submit score: {:?}", result.err().unwrap());
                    return;
                };

                info!("Submitted score");

                let mut settings = world.get_resource_mut::<Settings>().unwrap();
                settings.profile = Some(profile);
            },
        });

        commands.entity(entity).despawn_recursive();
        commands.entity(text_field).despawn_recursive()
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
                        poll_task::<Result<Profile, SubmitScoreError>>,
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
