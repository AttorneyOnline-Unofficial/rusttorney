use bb8::{Pool, PooledConnection, RunError};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{Error, NoTls};

pub struct DbWrapper {
    db_pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Clone for DbWrapper {
    fn clone(&self) -> Self {
        DbWrapper { db_pool: self.db_pool.clone() }
    }
}

impl DbWrapper {
    pub(crate) async fn get(
        &self,
    ) -> Result<
        PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        RunError<Error>,
    > {
        self.db_pool.get().await
    }

    pub fn new(db_pool: Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self { db_pool }
    }
}
