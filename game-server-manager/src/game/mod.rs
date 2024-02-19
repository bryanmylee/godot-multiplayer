mod get;
mod kill;
mod spawn;

use actix_web::web;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::RwLock;
use subprocess::Popen;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(get::list)
        .service(get::find_by_port)
        .service(spawn::spawn)
        .service(kill::kill);
}

#[derive(Debug)]
pub struct GamesData {
    games: RwLock<Games>,
}

impl GamesData {
    pub fn new() -> GamesData {
        GamesData {
            games: RwLock::new(Games::new()),
        }
    }
}

const BASE_PORT: u16 = 9000;
// TODO: configurable MAX_NUM_GAMES
const MAX_NUM_GAMES: usize = 250;

#[derive(Debug)]
pub struct Games([Option<Game>; MAX_NUM_GAMES]);

impl Games {
    fn new() -> Games {
        Games(std::array::from_fn(|_| None))
    }
}

impl Games {
    pub fn find_by_port(&self, port: u16) -> Option<&Game> {
        let game: Option<&Option<Game>> = self.0.get((port - BASE_PORT) as usize);
        let Some(game) = game else {
            return None;
        };
        game.into()
    }

    pub fn find_mut_by_port(&mut self, port: u16) -> &mut Option<Game> {
        self.0
            .get_mut((port - BASE_PORT) as usize)
            .expect("Failed to get mut game reference")
    }

    pub fn get_active_count(&self) -> usize {
        self.0.iter().filter(|p| p.is_some()).count()
    }

    pub fn get_all_active_description(&self) -> Vec<GameDescription> {
        self.0
            .iter()
            .filter_map(|game| match game {
                Some(game) => Some(game),
                None => None,
            })
            .map(|game| game.into())
            .collect()
    }

    pub fn get_available_entry(&mut self) -> Option<(u16, &mut Option<Game>)> {
        let idx = self.0.iter().position(|p| p.is_none());
        let Some(idx) = idx else {
            return None;
        };
        let game = self.0.get_mut(idx).unwrap();
        let idx: u16 = idx.try_into().unwrap();
        Some((idx + BASE_PORT, game))
    }
}

#[derive(Debug)]
pub struct Game {
    process: Popen,
    port: u16,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameDescription {
    pub process_id: u32,
    pub port: u16,
    pub created_at: DateTime<Utc>,
}

impl From<&Game> for GameDescription {
    fn from(value: &Game) -> Self {
        GameDescription {
            port: value.port,
            process_id: value.process.pid().expect("Failed to read process pid"),
            created_at: value.created_at,
        }
    }
}
