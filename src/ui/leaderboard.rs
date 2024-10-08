use bevy::prelude::*;

use crate::utils::{
    api::{ApiManager, PlayerScore, Token},
    tasks::TaskComponent,
};

use super::{fonts::FontsResource, theme::text_body_style};

#[derive(Component)]
struct Leaderboard;

#[derive(Clone)]
pub enum FetchLeaderboardRequest {
    NearScore { score: u32 },
    NearPlayer { token: Token },
    BestPlayers,
}

fn on_leaderboard_loaded(
    leaderboard_entity: Entity,
    request: FetchLeaderboardRequest,
    result: Result<Vec<PlayerScore>, reqwest::Error>,
    world: &mut World,
) {
    world
        .commands()
        .entity(leaderboard_entity)
        .despawn_descendants();
    let Some(font_res) = world.get_resource::<FontsResource>() else {
        return;
    };
    let font_res = font_res.clone();

    let mut commands = world.commands();

    let Ok(scores) = result else {
        commands.entity(leaderboard_entity).with_children(|c| {
            c.spawn(TextBundle::from_section(
                "Error loading leaderboard",
                text_body_style(&font_res),
            ));
        });
        return;
    };

    let scores = if let FetchLeaderboardRequest::NearScore { score } = request {
        let mut new_scores: Vec<_> = scores
            .iter()
            .cloned()
            .take_while(|v| v.score > score)
            .collect();
        new_scores.push(PlayerScore {
            score,
            player_name: t!("you").to_string(),
            rank: new_scores.last().map_or(0, |p| p.rank) + 1,
            id: 0,
        });
        let iter = scores
            .iter()
            .skip_while(|v| v.score > score)
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

    commands.entity(leaderboard_entity).with_children(|c| {
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
        let font_res = font_res.clone();

        let leaderboard = self
            .spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(20.)),
                        ..default()
                    },
                    ..default()
                },
                Leaderboard,
            ))
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("loading"),
                    text_body_style(&font_res),
                ));
            })
            .id();

        let request_clone = request.clone();

        self.spawn(TaskComponent::new(
            async move {
                info!("Loading leaderboard");
                match request {
                    FetchLeaderboardRequest::NearScore { score } => {
                        api_manager.fetch_leaderboard_by_score(score, count).await
                    }
                    FetchLeaderboardRequest::NearPlayer { token } => {
                        api_manager.fetch_leaderboard_by_player(&token, count).await
                    }
                    FetchLeaderboardRequest::BestPlayers => {
                        api_manager.fetch_best_players(count).await
                    }
                }
            },
            move |result, world| {
                info!("Leaderboard loaded");
                on_leaderboard_loaded(leaderboard, request_clone, result, world);
            },
        ));
    }
}
