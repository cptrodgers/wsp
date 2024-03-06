#[macro_use]
extern crate log;

pub mod server;

use crate::server::center::{SendWebhook, WSCenter};
use actix::SystemService;
use actix_web::middleware::Logger;
use actix_web::{guard, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::Deserialize;
use uuid::Uuid;

use crate::server::connector::Connector;

#[derive(Deserialize)]
struct QueryInfo {
    pub channel: Uuid,
}

async fn index_ws(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<QueryInfo>,
) -> Result<HttpResponse, Error> {
    ws::start(Connector::new(query.channel), &req, stream)
}

async fn handle_webhook_event(bytes: web::Bytes, query: web::Query<QueryInfo>) -> impl Responder {
    WSCenter::from_registry().do_send(SendWebhook {
        channel: query.channel,
        message: bytes,
    });

    "Ok"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::log_util::setup_logger().unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(
                web::resource("/ws")
                    .guard(guard::Header("upgrade", "websocket"))
                    .route(web::get().to(index_ws)),
            )
            .service(web::resource("/events").route(web::post().to(handle_webhook_event)))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
