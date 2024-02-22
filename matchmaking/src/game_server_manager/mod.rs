use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::GameServerManagerConfig;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct GameServerManagerDescription {
    pub process_id: u32,
    pub port: u16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameServerDescription {
    pub port: u16,
    pub host: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait GameServerManager: Sync {
    async fn spawn_new_game_server(
        &self,
    ) -> Result<GameServerDescription, Box<dyn std::error::Error>>;
}

pub struct RealGameServerManager {
    config: GameServerManagerConfig,
}

impl RealGameServerManager {
    pub fn new(config: GameServerManagerConfig) -> RealGameServerManager {
        RealGameServerManager { config }
    }
}

#[async_trait::async_trait]
impl GameServerManager for RealGameServerManager {
    async fn spawn_new_game_server(
        &self,
    ) -> Result<GameServerDescription, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{}/game/spawn", self.config.url))
            .header("Service-Key", &self.config.service_key)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        let spawned_server: GameServerManagerDescription = match resp.status() {
            StatusCode::CREATED => resp
                .json()
                .await
                .expect("Failed to parse game server description"),
            _ => {
                let text = resp.text().await?;
                return Err(text.into());
            }
        };

        Ok(GameServerDescription {
            host: self.config.game_server_external_host.clone(),
            port: spawned_server.port,
            created_at: spawned_server.created_at,
        })
    }
}
