#![allow(unused)]
use env_logger::Env;
use log::LevelFilter;
use rusttorney::master_server_client::MasterServerClient;
use rusttorney::{config::Config, server::AOServer};
use sqlx::SqlitePool;
use std::env;
use std::path::PathBuf;

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

    let mut pool = SqlitePool::new(
        &env::var("DATABASE_URL").unwrap_or("sqlite:storage/db.sqlite3".into()),
    )
    .await?;
    AOServer::new(&config, pool)?.run().await
}
