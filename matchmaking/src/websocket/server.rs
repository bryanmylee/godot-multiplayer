use super::session::ClientToServerMessage;
use crate::{
    config::MatchmakingConfig,
    game_server_manager::{GameServerDescription, GameServerManager},
    queue::{QueueData, QueueStatus},
};
use actix::{
    dev::ContextFutureSpawner, fut, Actor, ActorFutureExt, AsyncContext, Context, Handler,
    Recipient, WrapFuture,
};
use actix_web::{error, web};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

#[derive(Debug, Clone, actix::Message, Serialize)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClientMessage {
    StartGame(GameServerDescription),
}

type Sessions = HashMap<Uuid, Recipient<ServerToClientMessage>>;

#[derive(Clone)]
pub struct WebsocketServer {
    sessions: Arc<RwLock<Sessions>>,
    queue_data: web::Data<QueueData>,
    matchmaking_config: MatchmakingConfig,
    game_server_manager: web::Data<dyn GameServerManager>,
}

impl WebsocketServer {
    pub fn new(
        queue_data: web::Data<QueueData>,
        matchmaking_config: MatchmakingConfig,
        game_server_manager: web::Data<dyn GameServerManager>,
    ) -> Self {
        WebsocketServer {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            queue_data,
            matchmaking_config,
            game_server_manager,
        }
    }

    fn check_queue(&self, ctx: &mut Context<Self>) -> Result<(), error::Error> {
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
            ctx.address()
                .try_send(StartGame)
                .map_err(error::ErrorInternalServerError)?;
        }

        Ok(())
    }
}

async fn start_game(server: WebsocketServer) -> Result<(), error::Error> {
    let game_server = server.game_server_manager.spawn_new_game_server().await?;

    let ready_players = {
        let mut queue = server
            .queue_data
            .solo
            .write()
            .expect("Failed to get write lock on solo queue");
        queue.remove_ready_players(&server.matchmaking_config)?
    };

    {
        let sessions = server
            .sessions
            .read()
            .expect("Failed to get read lock on sessions");

        for player in ready_players {
            let Some(session) = sessions.get(&player.user_id) else {
                continue;
            };
            session.do_send(ServerToClientMessage::StartGame(game_server.clone()));
        }
    }

    Ok(())
}

impl Actor for WebsocketServer {
    type Context = Context<Self>;
}

impl Handler<ClientToServerMessage> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, message: ClientToServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        match message {
            ClientToServerMessage::Connect(recipient, uuid) => {
                let mut sessions = self
                    .sessions
                    .write()
                    .expect("Failed to get write lock on sessions");
                sessions.insert(uuid, recipient);
            }
            ClientToServerMessage::Disconnect(uuid) => {
                let mut sessions = self
                    .sessions
                    .write()
                    .expect("Failed to get write lock on sessions");
                sessions.remove(&uuid);
            }
        };
    }
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "Result<(), ()>")]
pub struct CheckQueue;
impl Handler<CheckQueue> for WebsocketServer {
    type Result = Result<(), ()>;

    fn handle(&mut self, _message: CheckQueue, ctx: &mut Self::Context) -> Self::Result {
        self.check_queue(ctx).map_err(|err| {
            println!("{err}");
        })
    }
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "Result<(), ()>")]
pub struct StartGame;
impl Handler<StartGame> for WebsocketServer {
    type Result = Result<(), ()>;

    fn handle(&mut self, _message: StartGame, ctx: &mut Self::Context) -> Self::Result {
        start_game(self.clone())
            .into_actor(self)
            .then(|res, _, _| {
                match res {
                    Err(err) => println!("{err}"),
                    Ok(()) => (),
                }
                fut::ready(())
            })
            .wait(ctx);

        Ok(())
    }
}
