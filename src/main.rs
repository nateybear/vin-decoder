use std::env::{self, args};

use dotenvy::dotenv_override;

use futures::task::waker;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use tracing::warn;

use anyhow::{Context, Result};
use vin_wmi::vin_wmi;

use std::time::Duration;

use csv::Writer;

mod decoding_item;
use decoding_item::DecodingItem;
mod element_attribute_value;
mod sp_vin_decode;
use sp_vin_decode::sp_vin_decode;
mod sp_vin_decode_core;
mod vin_descriptor;
mod vin_model_year;
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

fn load_env() {
    let _ = dotenv_override().map_or(None::<()>, |_| {
        warn!(".env file not found, falling back to environment variables");
        None
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    load_env();
    let vin: &str = &args().nth(1).unwrap_or("1N4BZ0CP6HC303730".to_string());
    tracing::debug!("VIN: {}", vin);
    let pool = build_pool().await?;
    tracing::debug!("{}", vin_descriptor(vin));
    tracing::debug!("{}", vin_wmi(vin));
    let mut conn = pool.acquire().await?;
    let mut wtr = Writer::from_writer(std::io::stdout());
    
    for r in sp_vin_decode(vin.into(), &mut conn).await? {
        wtr.serialize(r)?;
    }
    wtr.flush()?;

    Ok(())
}
