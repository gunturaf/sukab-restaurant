use std::env;

use deadpool_postgres::{Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use tokio_postgres::{Error, NoTls};

pub mod menu;
pub mod order;

#[derive(Debug)]
#[allow(dead_code)]
pub enum OperationError {
    FailedToConnect(PoolError),
    FailedToCreate(Error),
    FailedToGetDetail(Error),
    OtherError,
}

pub fn create_conn_pool() -> Pool {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config.host(
        env::var("PG_HOST")
            .unwrap_or("localhost".to_string())
            .as_str(),
    );
    pg_config.port(
        match env::var("PG_PORT") {
            Ok(v) => v.parse().unwrap_or(5432),
            Err(_) => 5432,
        }
    );
    pg_config.user(
        env::var("PG_USER")
            .unwrap_or("postgres".to_string())
            .as_str(),
    );
    pg_config.password(env::var("PG_PWD").unwrap_or("".to_string()).as_str());
    pg_config.dbname(
        env::var("PG_DBNAME")
            .unwrap_or("sukab_restaurant".to_string())
            .as_str(),
    );

    let mgr_cfg = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = Manager::from_config(pg_config, NoTls, mgr_cfg);
    // panic is OK here as to prevent runtime errors due to invalid postgres client pool:
    Pool::builder(mgr).max_size(10).build().unwrap()
}
