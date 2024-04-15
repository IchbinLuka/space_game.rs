use bevy::prelude::*;
use iyes_progress::ProgressCounter;

use crate::{
    states::{AppState, LoadingStateItem},
    ui::fonts::FontsResource, utils::misc::cleanup_system,
};

#[derive(Component)]
pub struct LoadingScreen;

#[derive(Component)]
struct ProgressBar;

fn loading_screen_setup(mut commands: Commands, font_res: Res<FontsResource>) {
    commands.spawn((Camera2dBundle::default(), LoadingScreen));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    align_content: AlignContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            LoadingScreen,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                t!("loading"),
                TextStyle {
                    font_size: 100.0,
                    color: Color::WHITE,
                    font: font_res.mouse_memoirs.clone(),
                },
            ));

            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(300.),
                    height: Val::Px(20.),
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                border_color: Color::WHITE.into(),
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    ProgressBar,
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(0.),
                            height: Val::Percent(100.),
                            ..default()
                        },
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                ));
            });
        });
}

fn loading_screen_progress(
    mut progress_bars: Query<&mut Style, With<ProgressBar>>,
    counter: Res<ProgressCounter>,
) {
    let progress = counter.progress_complete();
    let float_progress: f32 = progress.into();
    for mut style in &mut progress_bars {
        style.width = Val::Percent(float_progress * 100.0);
    }
}

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        for LoadingStateItem { loading_state, .. } in AppState::LOADING_STATES {
            app.add_systems(OnEnter(*loading_state), loading_screen_setup)
                .add_systems(OnExit(*loading_state), cleanup_system::<LoadingScreen>)
                .add_systems(
                    Update,
                    loading_screen_progress.run_if(in_state(*loading_state)),
                );
        }
    }
}
