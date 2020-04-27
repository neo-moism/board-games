use super::hall;
use actix::prelude::*;
use actix::Actor;
use actix::Addr;
use actix_web_actors::ws;
use std::time::Instant;

pub struct GameSession {
    id: usize,
    hb: Instant,
    game: String,
    name: String,
    addr: Addr<hall::Hall>,
}

impl GameSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(std::time::Duration::from_secs(1), |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > std::time::Duration::from_secs(10) {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                // notify chat server
                act.addr.do_send(hall::Disconnect(act.id));
                // stop actor
                ctx.stop();
                // don't try to send a ping
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
                    Ok(res) => act.id = res,
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
