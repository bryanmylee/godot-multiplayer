use super::server::{ServerMessage, WebsocketServer};
use crate::queue::QueueData;
use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Recipient, StreamHandler, WrapFuture,
};
use actix_web::web;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub enum ClientMessage {
    Connect(Recipient<ServerMessage>, Uuid),
    Disconnect(Uuid),
    CheckQueue(web::Data<QueueData>),
}

#[derive(Debug)]
pub struct WebsocketSession {
    /// Unique session id
    pub id: usize,

    /// The client must return a ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise the connection is dropped.
    pub last_heartbeat: Instant,

    pub user_id: Uuid,

    server_address: Addr<WebsocketServer>,
}

impl WebsocketSession {
    pub fn new(user_id: Uuid, server_address: Addr<WebsocketServer>) -> WebsocketSession {
        WebsocketSession {
            id: rand::random(),
            last_heartbeat: Instant::now(),
            user_id,
            server_address,
        }
    }

    fn mark_presence(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    fn has_timed_out(&self) -> bool {
        Instant::now().duration_since(self.last_heartbeat) > CLIENT_TIMEOUT
    }

    /// Sends a ping to the client every 5 seconds (HEARTBEAT_INTERVAL) and
    /// check whether the client has timed out.
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |session, ctx| {
            if session.has_timed_out() {
                println!("Client {} failed heartbeat, disconnecting", session.id);

                session
                    .server_address
                    .do_send(ClientMessage::Disconnect(session.user_id.clone()));

                // Stop the actor.
                ctx.stop();

                return;
            }

            // Send a heartbeat.
            ctx.ping(b"");
        });
    }
}

impl Actor for WebsocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);

        let session_address = ctx.address();
        self.server_address
            .send(ClientMessage::Connect(
                session_address.recipient(),
                self.user_id,
            ))
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Err(err) = res {
                    println!("Error connecting: {err}");
                    ctx.stop();
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::prelude::Running {
        self.server_address
            .send(ClientMessage::Disconnect(self.user_id))
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Err(err) = res {
                    println!("Error disconnecting: {err}");
                    ctx.stop();
                }
                fut::ready(())
            })
            .wait(ctx);
        actix::Running::Stop
    }
}

impl Handler<ServerMessage> for WebsocketSession {
    type Result = ();

    fn handle(&mut self, message: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        match message {
            ServerMessage::Text(text) => ctx.text(text),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(ping)) => {
                self.mark_presence();
                ctx.pong(&ping);
            }
            Ok(ws::Message::Pong(_)) => self.mark_presence(),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                self.server_address
                    .do_send(ClientMessage::Disconnect(self.user_id));
                ctx.close(reason);
                ctx.stop();
            }
            Err(err) => {
                println!("Error handling message: {err}");
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
