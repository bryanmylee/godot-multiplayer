mod solo;

use actix_web::{error, web};
use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::Serialize;
use std::collections::BinaryHeap;
use std::sync::RwLock;
use std::{cmp::Reverse, sync::Arc};
use uuid::Uuid;

use crate::identity::IdentityService;
use crate::{
    config::{self, MatchmakingConfig},
    identity::RealIdentityService,
};

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let id_service = web::Data::from(Arc::new(RealIdentityService::new(
        config::IDENTITY_CONFIG.clone(),
    )) as Arc<dyn IdentityService>);
    cfg.app_data(id_service)
        .service(web::scope("/solo").configure(solo::config_service));
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

trait BinaryHeapExt<T> {
    fn remove<F>(&mut self, f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool;
}

impl<T> BinaryHeapExt<T> for BinaryHeap<T>
where
    T: Eq + Ord + Clone,
{
    fn remove<F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        let mut removed: Option<T> = None;
        self.retain(|e| {
            let matches = f(e);
            if matches {
                removed = Some(e.clone());
            }
            !matches
        });
        removed
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Derivative, Serialize)]
#[derivative(PartialOrd, Ord)]
pub struct QueuedPlayer {
    joined_at: Reverse<DateTime<Utc>>,
    #[derivative(PartialOrd = "ignore", Ord = "ignore")]
    user_id: Uuid,
}

impl SoloQueue {
    pub fn contains_player(&self, user_id: &Uuid) -> bool {
        self.queue.iter().any(|p| &p.user_id == user_id)
    }

    pub fn join_queue(&mut self, user_id: Uuid) -> Result<QueuedPlayer, error::Error> {
        if self.contains_player(&user_id) {
            return Err(error::ErrorBadRequest(
                "Cannot join queue that was already joined",
            ));
        }
        let player = QueuedPlayer {
            user_id,
            joined_at: Reverse(Utc::now()),
        };
        self.queue.push(player.clone());
        Ok(player)
    }

    pub fn leave_queue(&mut self, user_id: &Uuid) -> Result<QueuedPlayer, error::Error> {
        match self.queue.remove(|p| &p.user_id == user_id) {
            Some(removed) => Ok(removed),
            None => Err(error::ErrorBadRequest(
                "Cannot leave queue that was not joined",
            )),
        }
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

            queue.queue.push(QueuedPlayer {
                joined_at: Reverse(Utc::now() - Duration::seconds(2)),
                user_id: Uuid::new_v4(),
            });

            let config = MatchmakingConfig {
                solo_game_desired_max_wait_time: Duration::seconds(5),
                solo_game_min_size: 2,
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
                _ = queue.join_queue(Uuid::new_v4());
            }

            let config = MatchmakingConfig {
                solo_game_desired_size: 4,
                ..MatchmakingConfig::default()
            };

            assert_eq!(queue.status(&config), QueueStatus::Ready);
        }
    }
}