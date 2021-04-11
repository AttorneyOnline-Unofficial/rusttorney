use std::net::IpAddr;
use tokio_postgres::{Error, NoTls};
use deadpool_postgres::{Pool, ClientWrapper, PoolError, Client};
use deadpool::managed::Object;

pub struct DbWrapper {
    db_pool: Pool,
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
        Client,
        PoolError,
    > {
        self.db_pool.get().await
    }

    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    pub async fn ipid(&self, ip: IpAddr) -> Result<i64, anyhow::Error> {
        let ip_str = ip.to_string();
        let mut conn = self.get().await?;
        {
            let tx = conn.transaction().await?;
            tx.execute("INSERT INTO ipids (ip_address) VALUES ($1) ON CONFLICT DO NOTHING", &[&ip_str]).await?;
            tx.commit().await?;
        }
        let ipid = conn
            .query_one(
                "SELECT ipid FROM ipids WHERE ip_address = $1",
                &[&ip_str],
            )
            .await?;
        Ok(ipid.get::<_, i64>(0_usize))
    }
}
