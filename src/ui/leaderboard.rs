use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, IoTaskPool, Task},
};

use crate::utils::api::{ApiManager, PlayerScore};

use super::{fonts::FontsResource, theme::text_body_style};

#[derive(Component)]
struct Leaderboard;

#[derive(Component)]
pub struct FetchLeaderboardTask(Task<Result<Vec<PlayerScore>, reqwest::Error>>);
impl FetchLeaderboardTask {
    fn spawn(api_manager: ApiManager, score: u32, num_players: u32) -> Self {
        let task_pool = IoTaskPool::get();
        let task = task_pool.spawn(async move {
            async_compat::Compat::new(api_manager.fetch_leaderboard(score, num_players)).await
        });

        Self(task)
    }
}

fn poll_leaderboard_status(
    mut leader_boards: Query<(&mut FetchLeaderboardTask, Entity), With<Leaderboard>>,
    mut commands: Commands,
    font_res: Res<FontsResource>,
) {
    for (mut task, entity) in &mut leader_boards {
        let FetchLeaderboardTask(ref mut task) = *task;
        let Some(result) = block_on(future::poll_once(task)) else {
            continue;
        };

        commands
            .entity(entity)
            .despawn_descendants()
            .remove::<FetchLeaderboardTask>();

        let Ok(scores) = result else {
            commands.entity(entity).with_children(|c| {
                c.spawn(TextBundle::from_section(
                    "Error loading leaderboard",
                    text_body_style(&font_res),
                ));
            });
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
        // TODO: Remove the loading text and display the leaderboard
    }
}

pub trait AddLeaderboardExtension {
    fn add_leaderboard(
        &mut self,
        focus_score: u32,
        count: u32,
        api_manager: ApiManager,
        font_res: &FontsResource,
    );
}

impl AddLeaderboardExtension for ChildBuilder<'_> {
    fn add_leaderboard(
        &mut self,
        focus_score: u32,
        count: u32,
        api_manager: ApiManager,
        font_res: &FontsResource,
    ) {
        let fetch_players_task = FetchLeaderboardTask::spawn(api_manager, focus_score, count);
        let font_res = font_res.clone();

        self.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
            fetch_players_task,
            Leaderboard,
        ))
        .with_children(|c| {
            c.spawn(TextBundle::from_section(
                t!("loading"),
                text_body_style(&font_res),
            ));
        });
    }
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, poll_leaderboard_status);
    }
}
