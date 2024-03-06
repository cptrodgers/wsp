#[macro_use]
extern crate log;

pub mod server;

use actix_web::middleware::Logger;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, guard};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::log_util::setup_logger().unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(
                web::resource("/ws")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .route(web::get().to(index_ws))
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
