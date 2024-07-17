use std::env;

use actix_web::{middleware::Logger, App, HttpServer};

mod order;

fn get_host_port() -> (String, u16) {
    const DEFAULT_PORT: u16 = 8080;

    let host_env = env::var("HTTP_HOST").unwrap_or("127.0.0.1".to_string());
    let port_env: u16 = match env::var("HTTP_PORT").ok() {
        Some(p) => p.parse().unwrap_or(DEFAULT_PORT),
        None => DEFAULT_PORT,
    };
    (host_env, port_env)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initialize global logger:
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().wrap(logger).service(order::service())
    })
    .bind(get_host_port())?
    .run()
    .await
}
