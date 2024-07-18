use async_trait::async_trait;
use deadpool_postgres::Pool;
use mockall::automock;
use postgres_from_row::FromRow;

use super::OperationError;

#[automock]
#[async_trait]
/// Menu repository abstraction.
/// Use this trait as dependency to make the usecase function be easy testable via mocks.
pub trait Repository {
    async fn get_by_id(&self, id: i64) -> Result<Menu, OperationError>;
}

#[derive(FromRow)]
pub struct Menu {
    #[from_row(rename = "menu_id")]
    pub id: i64,
    pub name: String,
}

#[cfg(test)]
impl Menu {
    pub fn new(id: i64, name: String) -> Self {
        Self { id, name }
    }
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
                let query = "SELECT * FROM menus WHERE menu_id = $1";
                conn.query_one(query, &[&id])
                    .await
                    .map(|row| Menu::from_row(&row))
                    .map_err(|e| OperationError::FailedToCreate(e))
            }
        }
    }
}
