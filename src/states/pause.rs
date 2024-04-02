use crate::{
    states::{game_running, AppState},
    ui::{
        button::TextButtonBundle,
        fonts::FontsResource,
        settings::{SettingsButton, SettingsScreen},
        theme::text_button_style,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;

use super::{game_paused, pause_physics, resume_physics, PausedState};

#[derive(Component)]
pub struct PauseScreen;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct QuitButton;

fn on_pause(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut commands: Commands,
    font_res: Res<FontsResource>,
) {
    pause_physics(&mut rapier_config);

    commands
        .spawn((
            PauseScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute, 
                    height: Val::Percent(100.),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(50.)),
                    ..default()
                },
                background_color: Color::rgba(0., 0., 0., 0.5).into(),
                ..default()
            },
        ))
        .with_children(|c| {
            let text_style = text_button_style(&font_res);

            c.spawn(TextBundle {
                text: Text::from_section(
                    t!("game_paused"),
                    TextStyle {
                        font_size: 70.,
                        ..text_style.clone()
                    },
                ),
                style: Style {
                    margin: UiRect {
                        bottom: Val::Px(50.),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            });

            c.spawn((
                TextButtonBundle::from_section(t!("settings"), text_style.clone()),
                SettingsButton,
            ));

            c.spawn((
                TextButtonBundle::from_section(t!("quit"), text_style.clone()),
                QuitButton,
            ));

            c.spawn((
                TextButtonBundle::from_section(t!("resume"), text_style.clone()),
                ResumeButton,
            ));
        });
}

fn resume_button(
    mut next_state: ResMut<NextState<PausedState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(PausedState::Running);
        }
    }
}

fn quit_button(
    mut app_state: ResMut<NextState<AppState>>,
    mut pause_state: ResMut<NextState<PausedState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            pause_state.set(PausedState::Running);
            app_state.set(AppState::StartScreenLoading);
        }
    }
}

fn on_resume(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut commands: Commands,
    loading_screen: Query<Entity, With<PauseScreen>>,
) {
    resume_physics(&mut rapier_config);

    for entity in &loading_screen {
        commands.entity(entity).despawn_recursive();
    }
}

fn pause_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PausedState>>,
    current_state: Res<State<PausedState>>,
    settings_screen: Query<(), With<SettingsScreen>>,
) {
    if settings_screen.get_single().is_ok() {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(if current_state.get() == &PausedState::Paused {
            PausedState::Running
        } else {
            PausedState::Paused
        });
    }
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                pause_game.run_if(game_running().or_else(game_paused())),
                (resume_button, quit_button).run_if(game_paused()),
            ),
        )
        .add_systems(OnExit(PausedState::Paused), on_resume)
        .add_systems(OnEnter(PausedState::Paused), on_pause);
    }
}
