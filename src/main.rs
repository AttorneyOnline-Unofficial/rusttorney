#![allow(unused)]
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use env_logger::Env;
use log::LevelFilter;
use rusttorney::networking::database::DbWrapper;
use rusttorney::networking::master_server_client::MasterServerClient;
use rusttorney::{config::Config, server::AOServer};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tokio_postgres::Config as PgConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter: &str;

    let config_path = PathBuf::from("./config/config.toml");
    let config_string = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_string)?;

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

    let pg_config =
        PgConfig::from_str("postgresql://postgres@localhost:5432/rusttorney")?;
    let pg_mgr =
        PostgresConnectionManager::new(pg_config, tokio_postgres::NoTls);
    let db = DbWrapper::new(Pool::builder().build(pg_mgr).await?);

    AOServer::new(config, db)?.run().await
}
