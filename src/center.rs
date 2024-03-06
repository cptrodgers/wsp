use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Supervised, SystemService};
use actix_web::web::Bytes;
use std::collections::HashMap;
use std::time::Duration;

use uuid::Uuid;

use crate::connector::{Connector, SendMessage};

#[derive(Default)]
pub struct WSCenter {
    pub items: HashMap<Uuid, Vec<Addr<Connector>>>,
}

impl Actor for WSCenter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(10), |act, _| {
            for senders in act.items.values_mut() {
                // Only keep connected sender
                senders.retain(|sender| sender.connected());
            }
        });
    }
}

impl Supervised for WSCenter {}
impl SystemService for WSCenter {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterConnector {
    pub channel: Uuid,
    pub connector: Addr<Connector>,
}

impl Handler<RegisterConnector> for WSCenter {
    type Result = ();

    fn handle(&mut self, msg: RegisterConnector, _: &mut Self::Context) -> Self::Result {
        if let Some(senders) = self.items.get_mut(&msg.channel) {
            senders.push(msg.connector);
        } else {
            self.items.insert(msg.channel, vec![msg.connector]);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendWebhook {
    pub channel: Uuid,
    pub message: Bytes,
}

impl Handler<SendWebhook> for WSCenter {
    type Result = ();
    fn handle(&mut self, msg: SendWebhook, _: &mut Self::Context) -> Self::Result {
        let SendWebhook { channel, message } = msg;
        if let Some(senders) = self.items.get(&channel) {
            for sender in senders {
                sender.do_send(SendMessage(message.clone()));
            }
        }
    }
}
