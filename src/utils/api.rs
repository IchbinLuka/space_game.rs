use bevy::prelude::{Deref, DerefMut, Resource};
use include_crypt::{include_crypt, EncryptedFile};
use serde::{Deserialize, Serialize};
use serde_json::json;
use space_game_common::{ScoreEvent, ScoreSubmission};

use crate::api_constants::API_URL;

static KEY_FILE: EncryptedFile = include_crypt!(".key");

#[derive(Deserialize, PartialEq)]
pub struct PlayerScore {
    pub score: u32,
    pub player_name: String,
    pub rank: u32,
}

#[derive(Deserialize, Serialize, Deref, DerefMut, Debug, Clone)]
pub struct Token(pub String);

impl From<Token> for String {
    fn from(token: Token) -> Self {
        token.0
    }
}

fn get_url(path: &str) -> String {
    format!("{}/{}", API_URL, path)
}

#[derive(Resource, Clone)]
pub struct ApiManager {
    client: reqwest::Client,
}

impl ApiManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_best_players(
        &self,
        num_players: u32,
    ) -> Result<Vec<PlayerScore>, reqwest::Error> {
        let url = format!("{}/ranking?num={}", API_URL, num_players);
        let response = self.client.get(url).send().await?.error_for_status()?;
        response.json::<Vec<PlayerScore>>().await
    }

    pub async fn fetch_leaderboard_by_score(
        &self,
        score: u32,
        num_players: u32,
    ) -> Result<Vec<PlayerScore>, reqwest::Error> {
        let url = format!(
            "{}/ranking_near_score?score={}&num={}",
            API_URL, score, num_players
        );
        let response = self.client.get(url).send().await?.error_for_status()?;
        response.json::<Vec<PlayerScore>>().await
    }

    pub async fn fetch_leaderboard_by_player(
        &self,
        token: &Token,
        num_players: u32,
    ) -> Result<Vec<PlayerScore>, reqwest::Error> {
        let url = format!("{}/ranking_near_self?num={}", API_URL, num_players);
        let response = self
            .client
            .get(url)
            .header("Authorization", token.as_str())
            .send()
            .await?
            .error_for_status()?;
        response.json::<Vec<PlayerScore>>().await
    }

    pub async fn create_player(&self, player_name: String) -> Result<Token, reqwest::Error> {
        let response = self
            .client
            .post(get_url("players"))
            .json(&json!({ "player_name": player_name }))
            .send()
            .await?
            .error_for_status()?;
        response.text().await.map(Token)
    }

    pub async fn submit_score(
        &self,
        score_events: &[ScoreEvent],
        auth_token: &Token,
    ) -> Result<(), reqwest::Error> {
        let key = KEY_FILE.decrypt();
        let boxed_slice = key.into_boxed_slice();
        let arr: Box<[u8; 16]> = boxed_slice
            .try_into()
            .expect("Invalid encryption key: Wrong size");

        let encrypted = ScoreSubmission::from_data(score_events, &arr).unwrap();

        self.client
            .post(get_url("scores"))
            .header("Authorization", auth_token.as_str())
            .body(encrypted.to_buffer())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
