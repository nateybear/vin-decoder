use std::env;

use dotenvy::dotenv_override;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use tracing::warn;

use anyhow::{Context, Result};

use std::time::Duration;

mod sp_vin_decode;
mod sp_vin_decode_core;
mod vin_descriptor;
mod vin_wmi;
use vin_descriptor::vin_descriptor;

async fn build_pool() -> Result<PgPool> {
    let pool_size: u32 = env::var("POOL_SIZE")
        .unwrap_or("4".to_owned())
        .parse()
        .context("Failed to parse POOL_SIZE environment variable")?;
    let options = PgConnectOptions::new(); // read config from environment!
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(60 * 60 * 24)) // your batch request will die after 24 hours
        .connect_with(options)
        .await
        .context("Failed to connect to postgres database")?;
    Ok(pool)
}

fn load_env() -> () {
    let _ = dotenv_override().map_or(None::<()>, |_| {
        warn!(".env file not found, falling back to environment variables");
        None
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    load_env();
    //    let pool = build_pool().await?;
    println!("{}", vin_descriptor("2T3E6RFV8MW017352"));
    Ok(())
}
