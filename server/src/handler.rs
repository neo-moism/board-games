use super::hall;
use actix::prelude::*;
use actix::Actor;
use actix::Addr;
use actix_web_actors::ws;
use std::time::Instant;

pub struct GameSession {
    pub id: usize,
    pub hb: Instant,
    pub _game: String,
    pub name: String,
    pub addr: Addr<hall::Hall>,
}

impl GameSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(std::time::Duration::from_secs(1), |act, ctx| {
            if Instant::now().duration_since(act.hb) > std::time::Duration::from_secs(10) {
                act.addr.do_send(hall::Disconnect(act.id));
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for GameSession {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let addr = ctx.address();
        self.addr
            .send(hall::Connect {
                addr: addr.recipient(),
                name: self.name.clone(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.id = res;
                        act.name = res.to_string()
                    }
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(hall::Disconnect(self.id));
        actix::Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<hall::StrMsg> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: hall::StrMsg, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
use actix::StreamHandler;

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => {
                ctx.stop();
                return;
            }
        };
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let segs: Vec<&str> = text.splitn(2, ' ').collect();
                if segs[0] == "/chat" {
                    self.addr.do_send(hall::HallMsg::Chat(hall::ChatMsg {
                        name: self.name.clone(),
                        content: segs.last().unwrap().to_string(),
                    }));
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
