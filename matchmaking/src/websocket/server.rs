use super::session::ClientMessage;
use actix::{Actor, Context, Handler, Recipient};
use actix_web::error;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub enum ServerMessage {
    Text(String),
}

pub struct WebsocketServer {
    sessions: HashMap<Uuid, Recipient<ServerMessage>>,
}

impl WebsocketServer {
    pub fn new() -> Self {
        WebsocketServer {
            sessions: HashMap::new(),
        }
    }

    pub fn send_message(&self, user_id: &Uuid, message: ServerMessage) -> Result<(), error::Error> {
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

impl Handler<ClientMessage> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, message: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        match message {
            ClientMessage::Connect(recipient, uuid) => {
                self.sessions.insert(uuid, recipient);
            }
            ClientMessage::Disconnect(uuid) => {
                self.sessions.remove(&uuid);
            }
        };
    }
}
