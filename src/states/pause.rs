use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierConfiguration;
use crate::{states::{game_running, AppState}, ui::fonts::FontsResource};

use super::game_paused;

#[derive(Component)]
pub struct PauseScreen;

#[derive(Component)]
pub struct ResumeButton;

fn on_pause(
    mut rapier_config: ResMut<RapierConfiguration>, 
    mut commands: Commands, 
    font_res: Res<FontsResource>, 
) {
    rapier_config.physics_pipeline_active = false;
    rapier_config.query_pipeline_active = false;

    commands.spawn((
        PauseScreen,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Flex, 
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(50.)), 
                ..default()
            }, 
            background_color: Color::rgba(0., 0., 0., 0.5).into(), 
            ..default()
        }
    )).with_children(|c| {
        c.spawn(TextBundle {
                text: Text::from_section(
                    "Paused", 
                    TextStyle {
                        font_size: 70., 
                        color: Color::WHITE, 
                        font: font_res.mouse_memoirs.clone(), 
                    }, 
                ), 
                ..default()
            });

        c.spawn((
            TextBundle {
                text: Text::from_section(
                    "Resume", 
                    TextStyle {
                        font_size: 50., 
                        color: Color::WHITE, 
                        font: font_res.mouse_memoirs.clone(), 
                    }, 
                ), 
                ..default()
            }, 
            ResumeButton, 
            Interaction::default(), 
        ));
    });
}

fn resume_button(
    mut next_state: ResMut<NextState<AppState>>, 
    mut query: Query<(&Interaction, &mut Text), (Changed<Interaction>, With<ResumeButton>)>, 
) {
    for (interaction, mut text) in &mut query {
        let Some(section) = text.sections.get_mut(0) else {
            error!("No text style found");
            continue;
        };
        match interaction {
            Interaction::Pressed => {
                next_state.set(AppState::MainScene);
            }, 
            Interaction::None => {
                section.style.color = Color::WHITE;
            }, 
            Interaction::Hovered => {
                section.style.color = Color::GRAY;
            },
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
    current_state: Res<State<AppState>>
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(
            if current_state.get() == &AppState::MainPaused {
                AppState::MainScene
            } else {
                AppState::MainPaused
            }
        );
    }
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update, 
                (
                    pause_game.run_if(
                        game_running().or_else(game_paused())
                    ), 
                    resume_button.run_if(game_paused()), 
                )
            )
            .add_systems(OnExit(AppState::MainPaused), on_resume)
            .add_systems(OnEnter(AppState::MainPaused), on_pause);
    }
}