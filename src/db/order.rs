use async_trait::async_trait;
use deadpool_postgres::{GenericClient, Pool};
use time::OffsetDateTime;
use tokio_postgres::types::ToSql;

use super::OperationError;

#[async_trait]
/// Order repository abstraction.
/// Use this trait as dependency to make the usecase function be easy testable via mocks.
pub trait Repository {
    /// Store the Order entity into the datastore.
    async fn create_order(&self, data: Order) -> Result<Order, OperationError>;
}

/// Represents a single Order entity.
pub struct Order {
    pub order_id: i64,
    pub table_number: i32,
    pub menu_id: i32,
    pub cook_time: i32,
    pub created_at: OffsetDateTime,
}

impl Order {
    /// Create a new Order entity to be used later for creation/deletion.
    pub fn new(table_number: i32, menu_id: i32, cook_time: i32) -> Self {
        Self {
            order_id: 0,
            table_number,
            menu_id,
            cook_time,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Clone)]
// Concrete implementation of Order repository
// which uses PostgreSQL as its datastore.
pub struct OrderRepository {
    db_pool: Pool,
}

impl OrderRepository {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl Repository for OrderRepository {
    async fn create_order(&self, data: Order) -> Result<Order, OperationError> {
        match self.db_pool.get().await {
            Err(e) => {
                return Err(OperationError::FailedToConnect(e));
            }
            Ok(conn) => {
                let insert_params: &[&(dyn ToSql + Sync)] = &[
                    &data.menu_id,
                    &data.table_number,
                    &data.cook_time,
                    &data.created_at,
                ];
                let query = "INSERT INTO orders (order_id, menu_id, table_number, cook_time, created_at) VALUES (DEFAULT, $1, $2, $3, $4) RETURNING order_id";
                conn.query_one(query, insert_params)
                    .await
                    .map(|row| {
                        let order_id: i64 = row.try_get("order_id").unwrap_or(0);
                        Order { order_id, ..data }
                    })
                    .map_err(|e| {
                        OperationError::FailedToCreate(e)
                    })
            }
        }
    }
}
