use std::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct PlayersData {
    pub queued: RwLock<Vec<Player>>,
}

impl PlayersData {
    pub fn new() -> PlayersData {
        PlayersData {
            queued: RwLock::new(vec![]),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub user_id: Uuid,
}
