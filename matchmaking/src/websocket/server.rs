use super::{session::ClientToServerMessage, RouteToActorMessage};
use actix::{Actor, Context, Handler, Recipient};
use actix_web::error;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub enum ServerToClientMessage {
    Text(String),
}

pub struct WebsocketServer {
    sessions: HashMap<Uuid, Recipient<ServerToClientMessage>>,
}

impl WebsocketServer {
    pub fn new() -> Self {
        WebsocketServer {
            sessions: HashMap::new(),
        }
    }

    pub fn send_message(
        &self,
        user_id: &Uuid,
        message: ServerToClientMessage,
    ) -> Result<(), error::Error> {
        let Some(recipient) = self.sessions.get(user_id) else {
            return Err(error::ErrorInternalServerError("Peer not found"));
        };
        recipient
            .try_send(message)
            .map_err(error::ErrorInternalServerError)
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

impl Handler<RouteToActorMessage> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, message: RouteToActorMessage, _ctx: &mut Self::Context) -> Self::Result {
        match message {
            RouteToActorMessage::CheckQueue(queue_data, config) => {}
        };
    }
}