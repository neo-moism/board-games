use actix::{Actor, Addr};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::time::Instant;

mod hall;
mod handler;

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Addr<hall::Hall>>,
) -> Result<HttpResponse, Error> {
    let s = handler::GameSession {
        id: 0,
        hb: Instant::now(),
        name: "".to_string(),
        gomoku_room: None,
        addr: data.get_ref().clone(),
    };
    let resp = ws::start(s, &req, stream);
    println!("{:?}", resp);
    resp
}

async fn hello(req: HttpRequest, _data: web::Data<Addr<hall::Hall>>) -> impl actix_web::Responder {
    req.path().to_owned()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let addr = hall::Hall::default().start();
    HttpServer::new(move || {
        App::new()
            .data(addr.clone())
            .wrap(actix_web::middleware::Logger::default())
            .route("/ws/", web::get().to(index))
            .route("/hello/", web::get().to(hello))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
