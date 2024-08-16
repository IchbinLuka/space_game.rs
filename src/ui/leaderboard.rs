use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, IoTaskPool, Task},
};

use crate::utils::api::{ApiManager, PlayerScore, Token};

use super::{fonts::FontsResource, theme::text_body_style};

#[derive(Component)]
struct Leaderboard;

#[derive(Clone)]
pub enum FetchLeaderboardRequest {
    NearScore { score: u32 },
    NearPlayer { token: Token },
    BestPlayers,
}

#[derive(Component)]
pub struct FetchLeaderboardTask {
    task: Task<Result<Vec<PlayerScore>, reqwest::Error>>,
    request: FetchLeaderboardRequest,
}

impl FetchLeaderboardTask {
    fn spawn(api_manager: ApiManager, request: FetchLeaderboardRequest, num_players: u32) -> Self {
        let task_pool = IoTaskPool::get();

        let task = match &request {
            FetchLeaderboardRequest::NearScore { score } => {
                let score = *score;
                task_pool.spawn(async move {
                    async_compat::Compat::new(
                        api_manager.fetch_leaderboard_by_score(score, num_players),
                    )
                    .await
                })
            }
            FetchLeaderboardRequest::NearPlayer { token } => {
                let token = token.clone();
                task_pool.spawn(async move {
                    async_compat::Compat::new(
                        api_manager.fetch_leaderboard_by_player(&token, num_players),
                    )
                    .await
                })
            }
            FetchLeaderboardRequest::BestPlayers => task_pool.spawn(async move {
                async_compat::Compat::new(api_manager.fetch_best_players(num_players)).await
            }),
        };

        Self { task, request }
    }
}

fn poll_leaderboard_status(
    mut leader_boards: Query<(&mut FetchLeaderboardTask, Entity), With<Leaderboard>>,
    mut commands: Commands,
    font_res: Res<FontsResource>,
) {
    for (mut task, entity) in &mut leader_boards {
        let FetchLeaderboardTask { task, request } = &mut *task;
        let Some(ref mut result) = block_on(future::poll_once(task)) else {
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

        let scores = if let FetchLeaderboardRequest::NearScore { score } = request {
            let mut new_scores: Vec<_> = scores
                .iter()
                .cloned()
                .take_while(|v| v.score > *score)
                .collect();
            new_scores.push(PlayerScore {
                score: *score,
                player_name: t!("you").to_string(),
                rank: (new_scores.len() + 1) as u32,
                id: 0,
            });
            let iter = scores
                .iter()
                .skip_while(|v| v.score > *score)
                .map(|v| PlayerScore {
                    score: v.score,
                    player_name: v.player_name.clone(),
                    rank: v.rank + 1,
                    id: v.id,
                });
            // Insert scores after the player's score
            new_scores.extend(iter);
            new_scores
        } else {
            scores.clone()
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
                        score.player_name.clone(),
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
        request: FetchLeaderboardRequest,
        count: u32,
        api_manager: ApiManager,
        font_res: &FontsResource,
    );
}

impl AddLeaderboardExtension for ChildBuilder<'_> {
    fn add_leaderboard(
        &mut self,
        request: FetchLeaderboardRequest,
        count: u32,
        api_manager: ApiManager,
        font_res: &FontsResource,
    ) {
        let fetch_players_task = FetchLeaderboardTask::spawn(api_manager, request, count);
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
