use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;

use crate::states::{
    game_over, game_running, reset_physics_speed, slow_down_physics, AppState, DespawnOnCleanup,
};
use crate::ui::button::TextButtonBundle;
use crate::ui::fonts::FontsResource;
use crate::ui::theme::{fullscreen_center_style, text_button_style, text_title_style};

use super::game_hud::Score;

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct BackToMenuButton;

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
                    t!("score", score = score.0),
                    text_button_style(&font_res),
                )
            });

            c.spawn((
                TextButtonBundle::from_section(t!("restart"), text_button_style(&font_res)),
                RestartButton,
            ));

            c.spawn((
                TextButtonBundle::from_section(t!("back_to_menu"), text_button_style(&font_res)),
                BackToMenuButton,
            ));
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

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                game_over_events.run_if(game_running()),
                (restart_game, back_to_menu).run_if(game_over()),
            ),
        )
        .add_systems(OnEnter(AppState::GameOver), game_over_screen_setup)
        .add_event::<GameOverEvent>();
    }
}
