#![allow(unused)]
use deadpool_postgres::{
    Config as PgConfig, Manager, ManagerConfig, Pool, RecyclingMethod,
};
use env_logger::Env;
use log::LevelFilter;
use rusttorney_server::client_manager::ClientManager;
use rusttorney_server::master_server_client::MasterServerClient;
use rusttorney_server::networking::database::DbWrapper;
use rusttorney_server::{config::Config, server::AOServer};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter: &str;

    let config_path = PathBuf::from("./config/config.toml");
    let config_string = std::fs::read_to_string(&config_path)?;
    let config: Arc<Config> = Arc::new(toml::from_str(&config_string)?);

    if config.debug {
        filter = "debug"
    } else {
        filter = "info"
    }

    env_logger::from_env(Env::default().default_filter_or(filter)).init();

    // tokio::spawn(async move {
    //     let config_path = PathBuf::from("./config/config.toml");
    //     let config_string = std::fs::read_to_string(&config_path).unwrap();
    //     let config: Config = toml::from_str(&config_string).unwrap();
    //     let mut master_server = MasterServerClient::from_config_with_connect(&config, "rusttorney").await.unwrap();
    //
    //     master_server.connection_loop().await.expect("MS connection loop panicked!");
    // });

    let mut pg_config = PgConfig::new();
    pg_config.dbname = Some("rusttorney".into());
    pg_config.user = Some("postgres".into());
    pg_config.manager =
        Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    let pool = pg_config.create_pool(tokio_postgres::NoTls)?;

    let db = DbWrapper::new(pool);

    AOServer::new(config, db)?.run().await
}
