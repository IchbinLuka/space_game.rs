use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::{block_on, futures_lite::future, Task};
use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_round_ui::autosize::RoundUiAutosizeMaterial;
use bevy_simple_text_input::TextInputBundle;

use crate::components::health::Health;
use crate::entities::space_station::SpaceStation;
use crate::model::settings::Settings;
use crate::states::{
    game_over, game_running, reset_physics_speed, slow_down_physics, AppState, DespawnOnCleanup,
};
use crate::ui::button::TextButtonBundle;
use crate::ui::fonts::FontsResource;
use crate::ui::theme::{fullscreen_center_style, text_button_style, text_title_style};
use crate::utils::api::{ApiManager, PlayerScore, Token};

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

#[derive(Component)]
struct SubmitButton;

#[derive(Component)]
enum Leaderboard {
    Loading(Task<Result<Vec<PlayerScore>, reqwest::Error>>),
    Loaded,
    Error,
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
    api_manager: Res<ApiManager>,
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

            c.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row, 
                    ..default()
                }, 
                ..default()
            }).with_children(|c| {
                c.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(200.), 
                            height: Val::Px(50.),
                            ..default()
                        }, 
                        // background_color: Color::RED.into(), 
                        ..default()
                    }, 
                    TextInputBundle::default()
                        .with_text_style(TextStyle {
                            font_size: 40., 
                            color: Color::WHITE,
                            ..default()
                        })
                        .with_inactive(true)
                        .with_text_style(text_button_style(&font_res))
                        .with_placeholder("Enter your Name", None)
                ));

                c.spawn((
                    TextButtonBundle::from_section("Submit", text_button_style(&font_res)), 
                    SubmitButton, 
                ));
            });
            let thread_pool = IoTaskPool::get();
            let score_value = score.value;
            let api_manager = api_manager.clone();
            let task: Task<Result<Vec<PlayerScore>, reqwest::Error>> =
                thread_pool.spawn(async move {
                    async_compat::Compat::new(api_manager.fetch_leaderboard(score_value)).await
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


#[derive(Component)]
pub struct CreatePlayerTask(Task<Result<Token, reqwest::Error>>);

#[derive(Component)]
pub struct SubmitScoreTask(Task<Result<(), reqwest::Error>>);



fn submit_score(
    submit_button: Query<&Interaction, (Changed<Interaction>, With<SubmitButton>)>,
    mut commands: Commands,
    api_manager: Res<ApiManager>,
) {
    for interaction in &submit_button {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let task_pool = IoTaskPool::get();
        let api_manager = api_manager.clone();
        let task = task_pool.spawn(async move {
            let player_name = "player".to_string();
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
    api_manager: Res<ApiManager>,
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
        let task_pool = IoTaskPool::get();
        let api_manager = api_manager.clone();
        let events = score.events.clone();
        let task = task_pool.spawn(async move {
            async_compat::Compat::new(api_manager.submit_score(&events, &token)).await
        });

        commands.spawn(SubmitScoreTask(task));
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
                        poll_task_status, 
                        submit_score,
                        poll_player_creation,
                        poll_submit_score,
                    ).run_if(game_over()),
                ),
            )
            .add_systems(OnEnter(AppState::GameOver), game_over_screen_setup)
            .add_event::<GameOverEvent>();
    }
}
