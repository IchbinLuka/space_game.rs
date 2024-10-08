use bevy::prelude::*;
use include_crypt::{include_crypt, EncryptedFile};
use serde::{Deserialize, Serialize};
use serde_json::json;
use space_game_common::{ScoreEvent, ScoreSubmission};

use crate::{api_constants::API_URL, model::settings::Profile};

static KEY_FILE: EncryptedFile = include_crypt!(".key");

#[derive(Deserialize, PartialEq, Clone)]
pub struct PlayerScore {
    pub score: u32,
    pub player_name: String,
    pub rank: u32,
    pub id: u32,
}

#[derive(Deserialize)]
struct SelfResponse {
    id: u32,
    player_name: String,
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ApiError {
    NotAuthenticated,
    NetworkError(String),
    InvalidResponse(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError::NetworkError(error.to_string())
    }
}

impl ApiManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_profile(&self, token: &Token) -> Result<Profile, ApiError> {
        let url = get_url("self");
        let response = self
            .client
            .get(url)
            .header("Authorization", token.0.clone())
            .send()
            .await?
            .error_for_status()
            .map_err(|_| ApiError::NotAuthenticated)?;
        let parsed = response
            .json::<SelfResponse>()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))?;
        Ok(Profile {
            id: parsed.id,
            name: parsed.player_name,
            token: token.clone(),
        })
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

    pub async fn create_player(&self, player_name: String) -> Result<Profile, ApiError> {
        let response = self
            .client
            .post(get_url("players"))
            .json(&json!({ "player_name": player_name }))
            .send()
            .await?
            .error_for_status()?;
        let token = response.text().await.map(Token)?;
        self.get_profile(&token).await
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
