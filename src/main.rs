use actix_web::{get, App, HttpResponse, HttpServer, Responder};

pub mod order;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Sukab Restaurant")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(order::service())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
