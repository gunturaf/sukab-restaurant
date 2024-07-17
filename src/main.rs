use actix_web::{middleware::Logger, App, HttpServer};

mod order;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().wrap(logger).service(order::service())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
