mod solo;

use actix_web::web;
use chrono::{DateTime, Utc};
use derivative::Derivative;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::config::MatchmakingConfig;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/solo").configure(solo::config_service));
}

#[derive(Debug)]
pub struct QueueData {
    pub solo: RwLock<SoloQueue>,
}

impl QueueData {
    pub fn new() -> QueueData {
        QueueData {
            solo: RwLock::new(SoloQueue::new()),
        }
    }
}

#[derive(Debug)]
pub struct SoloQueue {
    queue: BinaryHeap<QueuedPlayer>,
}

impl SoloQueue {
    fn new() -> SoloQueue {
        SoloQueue {
            queue: BinaryHeap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Derivative)]
#[derivative(PartialOrd, Ord)]
struct QueuedPlayer {
    joined_at: Reverse<DateTime<Utc>>,
    #[derivative(PartialOrd = "ignore", Ord = "ignore")]
    user_id: Uuid,
}

impl SoloQueue {
    pub fn insert_player(&mut self, user_id: Uuid) {
        self.queue.push(QueuedPlayer {
            user_id,
            joined_at: Reverse(Utc::now()),
        })
    }

    pub fn remove_player(&mut self, user_id: &Uuid) {
        self.queue.retain(|p| &p.user_id != user_id);
    }

    pub fn status(&self, config: &MatchmakingConfig) -> QueueStatus {
        if (self.queue.len() as u8) >= config.solo_game_desired_size {
            return QueueStatus::Ready;
        }

        let Some(oldest_player) = self.queue.peek() else {
            return QueueStatus::NotReady;
        };

        if Utc::now().signed_duration_since(oldest_player.joined_at.0)
            > config.solo_game_desired_max_wait_time
        {
            return if (self.queue.len() as u8) >= config.solo_game_min_size {
                QueueStatus::LongWaitReady
            } else {
                QueueStatus::LongWaitNotReady
            };
        }

        QueueStatus::NotReady
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueStatus {
    NotReady,
    Ready,
    LongWaitNotReady,
    LongWaitReady,
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    mod status {
        use super::*;

        #[test]
        fn empty_queue_is_not_ready() {
            let queue = SoloQueue::new();
            let config = MatchmakingConfig::default();
            assert_eq!(queue.status(&config), QueueStatus::NotReady);
        }

        #[test]
        fn less_than_min_players_and_long_wait_is_long_wait_not_ready() {
            let mut queue = SoloQueue::new();

            queue.queue.push(QueuedPlayer {
                joined_at: Reverse(Utc::now() - Duration::seconds(10)),
                user_id: Uuid::new_v4(),
            });

            let config = MatchmakingConfig {
                solo_game_desired_max_wait_time: Duration::seconds(5),
                solo_game_min_size: 2,
                ..MatchmakingConfig::default()
            };

            assert_eq!(queue.status(&config), QueueStatus::LongWaitNotReady);
        }

        #[test]
        fn more_than_min_players_and_long_wait_is_long_wait_ready() {
            let mut queue = SoloQueue::new();

            queue.queue.push(QueuedPlayer {
                joined_at: Reverse(Utc::now() - Duration::seconds(10)),
                user_id: Uuid::new_v4(),
            });

            let config = MatchmakingConfig {
                solo_game_desired_max_wait_time: Duration::seconds(5),
                solo_game_min_size: 1,
                ..MatchmakingConfig::default()
            };

            assert_eq!(queue.status(&config), QueueStatus::LongWaitReady);
        }

        #[test]
        fn short_wait_is_not_ready() {
            let mut queue = SoloQueue::new();

            queue.queue.push(QueuedPlayer {
                joined_at: Reverse(Utc::now() - Duration::seconds(10)),
                user_id: Uuid::new_v4(),
            });

            let config = MatchmakingConfig {
                solo_game_desired_max_wait_time: Duration::seconds(20),
                ..MatchmakingConfig::default()
            };

            assert_eq!(queue.status(&config), QueueStatus::NotReady);
        }

        #[test]
        fn enough_for_desired_players_is_ready() {
            let mut queue = SoloQueue::new();

            for _ in 0..4 {
                queue.insert_player(Uuid::new_v4());
            }

            let config = MatchmakingConfig {
                solo_game_desired_size: 4,
                ..MatchmakingConfig::default()
            };

            assert_eq!(queue.status(&config), QueueStatus::Ready);
        }
    }
}
