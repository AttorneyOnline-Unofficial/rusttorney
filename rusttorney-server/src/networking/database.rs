use bb8::{Pool, PooledConnection, RunError};
use bb8_postgres::PostgresConnectionManager;
use std::net::IpAddr;
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

    pub async fn ipid(&self, ip: IpAddr) -> Result<i32, anyhow::Error> {
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
        Ok(ipid.get::<_, i32>(0_usize))
    }

    pub async fn add_hdid(
        &mut self,
        hdid: String,
        ipid: u32,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.get().await?;
        let tx = conn.transaction().await?;
        let ipid = ipid as i32;

        tx.execute("INSERT INTO hdids (hdid, ipid) VALUES ($1, $2) ON CONFLICT DO NOTHING", &[&hdid, &ipid]).await?;
        tx.commit().await.map_err(|e| e.into())
    }
}
