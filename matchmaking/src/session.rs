use actix::{Actor, ActorContext, AsyncContext, Running, StreamHandler};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    /// Unique session id
    pub id: usize,

    /// The client must return a ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise the connection is dropped.
    pub last_heartbeat: Instant,

    pub user_id: Uuid,
}

impl WsSession {
    pub fn new(user_id: Uuid) -> WsSession {
        WsSession {
            id: rand::random(),
            last_heartbeat: Instant::now(),
            user_id,
        }
    }

    /// Sends a ping to the client every 5 seconds (HEARTBEAT_INTERVAL) and
    /// check for heartbeats from the client.
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |session, ctx| {
            // Check client heartbeats.
            if Instant::now().duration_since(session.last_heartbeat) > CLIENT_TIMEOUT {
                println!(
                    "Websocket client {} failed heartbeat, disconnecting",
                    session.id
                );

                // Stop the actor.
                ctx.stop();

                return;
            }

            // Send a server heartbeat.
            ctx.ping(b"");
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    /// Called on actor start.
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat process on session start.
        self.heartbeat(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let Ok(msg) = msg else {
            ctx.stop();
            return;
        };
        match msg {
            ws::Message::Ping(msg) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.last_heartbeat = Instant::now();
            }
            ws::Message::Text(text) => {
                let text = text.trim();
                println!("{text}");

                // let m = text.trim();
                // // we check for /sss type of messages
                // if m.starts_with('/') {
                //     let v: Vec<&str> = m.splitn(2, ' ').collect();
                //     match v[0] {
                //         "/list" => {
                //             // Send ListRooms message to chat server and wait for
                //             // response
                //             println!("List rooms");
                //             self.addr
                //                 .send(server::ListRooms)
                //                 .into_actor(self)
                //                 .then(|res, _, ctx| {
                //                     match res {
                //                         Ok(rooms) => {
                //                             for room in rooms {
                //                                 ctx.text(room);
                //                             }
                //                         }
                //                         _ => println!("Something is wrong"),
                //                     }
                //                     fut::ready(())
                //                 })
                //                 .wait(ctx)
                //             // .wait(ctx) pauses all events in context,
                //             // so actor wont receive any new messages until it get list
                //             // of rooms back
                //         }
                //         "/join" => {
                //             if v.len() == 2 {
                //                 self.room = v[1].to_owned();
                //                 self.addr.do_send(server::Join {
                //                     id: self.id,
                //                     name: self.room.clone(),
                //                 });

                //                 ctx.text("joined");
                //             } else {
                //                 ctx.text("!!! room name is required");
                //             }
                //         }
                //         "/name" => {
                //             if v.len() == 2 {
                //                 self.name = Some(v[1].to_owned());
                //             } else {
                //                 ctx.text("!!! name is required");
                //             }
                //         }
                //         _ => ctx.text(format!("!!! unknown command: {m:?}")),
                //     }
                // } else {
                //     let msg = if let Some(ref name) = self.name {
                //         format!("{name}: {m}")
                //     } else {
                //         m.to_owned()
                //     };
                //     // send message to chat server
                //     self.addr.do_send(server::ClientMessage {
                //         id: self.id,
                //         msg,
                //         room: self.room.clone(),
                //     })
                // }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
