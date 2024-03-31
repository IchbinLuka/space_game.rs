use bevy::prelude::*;

use crate::{states::{in_start_menu, AppState}, ui::{button::TextButtonBundle, fonts::FontsResource, theme::text_button_style}};


#[derive(Component)]
struct StartScreen;

#[derive(Component)]
struct StartButton;

fn setup_start_screen(font_res: Res<FontsResource>, mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), StartScreen));
    info!("Setting up start screen");
    commands
        .spawn((
            StartScreen,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn((
                StartButton,
                TextButtonBundle {
                    text_bundle: TextBundle::from_section("Start", text_button_style(&font_res)),
                    ..default()
                },
            ));
        });
}

fn start_screen_cleanup(mut commands: Commands, query: Query<Entity, With<StartScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn start_game(
    start_button: Query<&Interaction, (With<StartButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &start_button {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::MainSceneLoading);
        }
    }
}

pub struct StartScreenPlugin;

impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartScreen), setup_start_screen)
            .add_systems(OnExit(AppState::StartScreen), start_screen_cleanup)
            .add_systems(Update, (start_game,).run_if(in_start_menu()));
    }
}
