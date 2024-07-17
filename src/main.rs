use std::env;

use actix_web::{middleware::Logger, web, App, HttpServer};
use log;

mod db;
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

    let host_port = get_host_port();

    let db_conn_pool = db::create_conn_pool();
    log::info!("PostgreSQL connection pool is created: {:?}", db_conn_pool.status());

    let server = HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(web::Data::new(db_conn_pool.clone()))
            .service(order::service())
    })
    .bind(host_port.clone())?
    .run();

    log::info!("Server running at http://{}:{}/", host_port.0, host_port.1);

    server.await
}
