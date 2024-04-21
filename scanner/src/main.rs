mod db;
mod error;
mod scanner;
mod types;
extern crate core;
extern crate num_cpus;
use crate::db::PgStorage;
use crate::scanner::Scanner;
use bitcoincore_rpc::Auth;
use clap::Parser;
use error::Result;
use log::{error, info};
use sqlx::pool::PoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::time::Duration;

const DEFAULT_INTERVAL_SECS: u64 = 3; // 10min
const DEFAULT_RPC_RETRIES: usize = 3;
const DEFAULT_BATCH_SZE: usize = 8;

#[derive(Parser, Debug)]
struct Args {
    /// Pull single block
    #[arg(long)]
    pub single: bool,
    /// Block height to start scanning
    #[arg(long)]
    pub start: Option<u64>,
    /// Interval of scanning in seconds
    #[arg(long)]
    pub interval: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let btc_rpc_url = env::var("BTC_RPC_URL").expect("Not find `BTC_RPC_URL`");
    let btc_client = bitcoincore_rpc::Client::new(
        &btc_rpc_url,
        Auth::UserPass("cdd".to_string(), "csq123456".to_string()),
    )
    .expect("BTC client");
    info!("Node RPC: {}", btc_rpc_url);

    let db_url = env::var("DATABASE_URL").expect("Not find `DATABASE_URL`");
    let pool: Pool<Postgres> = PoolOptions::new()
        .connect(db_url.as_str())
        .await
        .expect("connect db failed");
    info!("Connecting db...ok");

    let storage = PgStorage::new(pool);
    let args = Args::parse();
    let start = if let Some(start) = args.start {
        start
    } else {
        storage.get_tip().await.unwrap_or(0)
    };
    let interval = if let Some(interval) = args.interval {
        Duration::from_secs(interval)
    } else {
        Duration::from_secs(DEFAULT_INTERVAL_SECS)
    };

    info!("Scanning interval: {}s", interval.as_secs());
    info!("Starting at block: {}", start);
    info!("Starting syncing...");
    let scanner = Scanner::new(DEFAULT_RPC_RETRIES, num_cpus::get(), btc_client, storage)
        .expect("failed to new scanner");
    let _ = scanner.run(start, interval, args.single).await;

    Ok(())
}
