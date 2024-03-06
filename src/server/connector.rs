// Reference: https://github.com/actix/examples/tree/master/websockets/echo/src
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web::Bytes;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::server::center::{RegisterConnector, WSCenter};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Connector {
    channel: Uuid,
    hb: Instant,
}

impl Connector {
    pub fn new(channel: Uuid) -> Self {
        Self {
            channel,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                info!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for Connector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        WSCenter::from_registry().do_send(RegisterConnector {
            channel: self.channel,
            connector: ctx.address(),
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Connector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub Bytes);

impl Handler<SendMessage> for Connector {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.0);
    }
}
