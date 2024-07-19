use std::{env, sync::Arc};

use actix_web::{middleware::Logger, web, App, HttpServer};
use sukab_resto::db::create_conn_pool;
use sukab_resto::db::menu::{MenuRepository, Repository as MenuRepositoryTrait};
use sukab_resto::db::order::{OrderRepository, Repository as OrderRepositoryTrait};
use log;
use sukab_resto::order::service;

/// get host:port pair for our HTTP server.
fn get_host_port() -> (String, u16) {
    const DEFAULT_PORT: u16 = 8080;

    let host_env = env::var("HTTP_HOST").unwrap_or("127.0.0.1".to_string());
    let port_env: u16 = match env::var("HTTP_PORT").ok() {
        Some(p) => p.parse().unwrap_or(DEFAULT_PORT),
        None => DEFAULT_PORT,
    };
    (host_env, port_env)
}

fn set_global_logger() {
    let rust_log_flag = "RUST_LOG";
    match env::var(rust_log_flag) {
        Ok(_) => {}
        Err(_) => env::set_var(rust_log_flag, "debug"),
    };
    env_logger::init();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    set_global_logger();

    let db_conn_pool = create_conn_pool();
    log::info!(
        "PostgreSQL connection pool is created: {:?}",
        db_conn_pool.clone()
    );

    let host_port = get_host_port();

    let server = HttpServer::new(move || {
        let logger = Logger::default();
        let order_repo = OrderRepository::new(db_conn_pool.clone());
        let arc_order_repo: Arc<dyn OrderRepositoryTrait> = Arc::new(order_repo);
        let menu_repo = MenuRepository::new(db_conn_pool.clone());
        let arc_menu_repo: Arc<dyn MenuRepositoryTrait> = Arc::new(menu_repo);
        App::new()
            .wrap(logger)
            .app_data(web::Data::from(arc_order_repo))
            .app_data(web::Data::from(arc_menu_repo))
            .service(service())
    })
    .bind(host_port.clone())?
    .run();

    log::info!("Server running at http://{}:{}/", host_port.0, host_port.1);

    server.await
}
