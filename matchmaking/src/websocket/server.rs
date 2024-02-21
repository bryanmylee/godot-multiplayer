use super::{session::ClientToServerMessage, RoutingToServerMessage};
use crate::{
    config::MatchmakingConfig,
    queue::{QueueData, QueueStatus},
};
use actix::{Actor, Context, Handler, Recipient};
use actix_web::{error, web};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, actix::Message, Serialize)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClientMessage {
    StartGame,
}

pub struct WebsocketServer {
    sessions: HashMap<Uuid, Recipient<ServerToClientMessage>>,
    queue_data: web::Data<QueueData>,
    matchmaking_config: MatchmakingConfig,
}

impl WebsocketServer {
    pub fn new(queue_data: web::Data<QueueData>, matchmaking_config: MatchmakingConfig) -> Self {
        WebsocketServer {
            sessions: HashMap::new(),
            queue_data,
            matchmaking_config,
        }
    }

    pub fn check_queue(&self) -> Result<(), error::Error> {
        let queue_ready = {
            let queue = self
                .queue_data
                .solo
                .read()
                .expect("Failed to get read lock on solo queue");

            match queue.status(&self.matchmaking_config) {
                QueueStatus::Ready | QueueStatus::LongWaitReady => true,
                _ => false,
            }
        };

        if queue_ready {
            self.start_game()?
        }

        Ok(())
    }

    pub fn start_game(&self) -> Result<(), error::Error> {
        let ready_players = {
            let mut queue = self
                .queue_data
                .solo
                .write()
                .expect("Failed to get write lock on solo queue");
            queue.remove_ready_players(&self.matchmaking_config)?
        };

        for player in ready_players {
            let Some(session) = self.sessions.get(&player.user_id) else {
                continue;
            };
            session.do_send(ServerToClientMessage::StartGame);
        }

        Ok(())
    }
}

impl Actor for WebsocketServer {
    type Context = Context<Self>;
}

impl Handler<ClientToServerMessage> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, message: ClientToServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        match message {
            ClientToServerMessage::Connect(recipient, uuid) => {
                self.sessions.insert(uuid, recipient);
            }
            ClientToServerMessage::Disconnect(uuid) => {
                self.sessions.remove(&uuid);
            }
        };
    }
}

impl Handler<RoutingToServerMessage> for WebsocketServer {
    type Result = ();

    fn handle(
        &mut self,
        message: RoutingToServerMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match message {
            RoutingToServerMessage::CheckQueue => match self.check_queue() {
                Ok(_) => (),
                Err(err) => println!("{err}"),
            },
        };
    }
}
