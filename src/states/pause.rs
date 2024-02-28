use crate::{
    states::{game_running, AppState},
    ui::{
        button::TextButtonBundle, fonts::FontsResource, settings::OpenSettings,
        theme::text_button_style,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;

use super::game_paused;

#[derive(Component)]
pub struct PauseScreen;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct SettingsButton;

fn on_pause(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut commands: Commands,
    font_res: Res<FontsResource>,
) {
    rapier_config.physics_pipeline_active = false;
    rapier_config.query_pipeline_active = false;

    commands
        .spawn((
            PauseScreen,
            NodeBundle {
                style: Style {
                    // width: Val::Percent(100.),
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
                TextButtonBundle::from_section(t!("resume"), text_style.clone()),
                ResumeButton,
            ));
        });
}

fn resume_button(
    mut next_state: ResMut<NextState<AppState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::MainScene);
        }
    }
}

fn settings_button(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.add(OpenSettings);
            break;
        }
    }
}

fn on_resume(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut commands: Commands,
    loading_screen: Query<Entity, With<PauseScreen>>,
) {
    rapier_config.physics_pipeline_active = true;
    rapier_config.query_pipeline_active = true;

    for entity in &loading_screen {
        commands.entity(entity).despawn_recursive();
    }
}

fn pause_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(if current_state.get() == &AppState::MainPaused {
            AppState::MainScene
        } else {
            AppState::MainPaused
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
                resume_button.run_if(game_paused()),
                settings_button.run_if(game_paused()),
            ),
        )
        .add_systems(OnExit(AppState::MainPaused), on_resume)
        .add_systems(OnEnter(AppState::MainPaused), on_pause);
    }
}
