use async_trait::async_trait;
use deadpool_postgres::{GenericClient, Object, Pool};
use mockall::automock;
use postgres_from_row::FromRow;
use postgres_types::ToSql;
use time::OffsetDateTime;

use super::OperationError;

#[automock]
#[async_trait]
/// Order repository abstraction.
/// Use this trait as dependency to make the usecase function be easy testable via mocks.
pub trait Repository {
    /// Store the Order entity into the datastore.
    async fn create_order(&self, data: Order) -> Result<Order, OperationError>;
    /// List Orders by Table number.
    async fn list_by_table(&self, table_number: i32) -> Result<Vec<Order>, OperationError>;
    /// Get Order detail by its ID and table_number.
    async fn get_order_detail(
        &self,
        table_number: i32,
        order_id: i64,
    ) -> Result<Option<Order>, OperationError>;
}

/// Represents a single Order entity.
#[derive(FromRow)]
pub struct Order {
    pub order_id: i64,
    pub table_number: i32,
    pub menu_id: i32,
    pub cook_time: i32,
    pub name: Option<String>,
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
            name: None,
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

impl OrderRepository {
    async fn get_conn(&self) -> Result<Object, OperationError> {
        self.db_pool
            .get()
            .await
            .map_err(|e| OperationError::FailedToConnect(e))
    }
}

#[async_trait]
impl Repository for OrderRepository {
    async fn create_order(&self, data: Order) -> Result<Order, OperationError> {
        let conn = self.get_conn().await?;

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
            .map_err(|e| OperationError::FailedToCreate(e))
    }

    async fn list_by_table(&self, table_number: i32) -> Result<Vec<Order>, OperationError> {
        let conn = self.get_conn().await?;

        let query = "SELECT o.*, m.* FROM orders o INNER JOIN menus m ON o.menu_id = m.menu_id WHERE table_number = $1 ORDER BY $2 ASC";
        conn.query(query, &[&table_number, &"created_at"])
            .await
            .map(|rows| {
                rows.iter()
                    .map(|row| Order::try_from_row(row).unwrap_or(Order::new(0, 0, 0)))
                    .collect::<Vec<Order>>()
            })
            .map_err(|e| OperationError::FailedToCreate(e))
    }

    async fn get_order_detail(
        &self,
        table_number: i32,
        order_id: i64,
    ) -> Result<Option<Order>, OperationError> {
        let conn = self.get_conn().await?;

        let query = "SELECT o.*, m.* FROM orders o INNER JOIN menus m ON o.menu_id = m.menu_id WHERE table_number = $1 AND order_id = $2 LIMIT 1";
        conn.query_opt(query, &[&table_number, &order_id])
            .await
            .map(|row| match row {
                Some(r) => Order::try_from_row(&r).map(|o| Some(o)).unwrap_or(None),
                None => None,
            })
            .map_err(|e| OperationError::FailedToGetDetail(e))
    }
}
