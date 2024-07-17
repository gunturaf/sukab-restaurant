use async_trait::async_trait;
use deadpool_postgres::Pool;

use super::OperationError;

#[async_trait]
/// Menu repository abstraction.
/// Use this trait as dependency to make the usecase function be easy testable via mocks.
pub trait Repository {
    async fn get_by_id(&self, id: i64) -> Result<Menu, OperationError>;
}

pub struct Menu {
    pub name: String,
}

#[derive(Clone)]
// Concrete implementation of menu repository
// which uses PostgreSQL as its datastore.
pub struct MenuRepository {
    db_pool: Pool,
}

impl MenuRepository {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl Repository for MenuRepository {
    async fn get_by_id(&self, id: i64) -> Result<Menu, OperationError> {
        match self.db_pool.get().await {
            Err(e) => {
                return Err(OperationError::FailedToConnect(e));
            }
            Ok(conn) => {
                let query = "SELECT name FROM menus WHERE menu_id = $1";
                conn.query_one(query, &[&id])
                    .await
                    .map(|row| {
                        Menu {
                            name: row.try_get("name").unwrap_or("--".to_string()),
                        }
                    })
                    .map_err(|e| {
                        OperationError::FailedToCreate(e)
                    })
            }
        }
    }
}
