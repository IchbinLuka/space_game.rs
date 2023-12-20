use bevy::prelude::*;

use crate::AppState;

#[derive(Component)]
pub struct LoadingScreen;

fn loading_screen_setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), LoadingScreen));

    commands.spawn((
        NodeBundle {
            style: Style {
                align_content: AlignContent::Center,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        }, 
        LoadingScreen
    )).with_children(|p| {
        p.spawn(
            TextBundle::from_section("Loading", TextStyle { font_size: 100.0,  color: Color::WHITE, ..default() })
        );
    });
}

fn loading_screen_cleanup(
    mut commands: Commands, 
    query: Query<Entity, With<LoadingScreen>>
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}


pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(OnEnter(AppState::Loading), loading_screen_setup)
            .add_systems(OnExit(AppState::Loading), loading_screen_cleanup);
    }
}