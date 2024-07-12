use std::{env, sync::Arc};

use axum::{
    routing::{get, post},
    serve, Router,
};
use dotenvy::dotenv_override;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use tracing_subscriber::{filter::EnvFilter, fmt};

use tokio::net::TcpListener;
use tracing::{info, warn};

use anyhow::{Context, Result};

use std::time::Duration;

mod element_attribute_value;
mod error;
mod handlers;
mod types;
use handlers::*;
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

    info!("Connecting to database with {} connections and configuration:\n\n{:?}\n", pool_size, options);

    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(60)) // your batch request will die after 1 minute
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
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .with_thread_ids(true)
        .init();

    load_env();
    let pool = build_pool().await?;

    let app = Router::new()
        .route("/decode/batch", post(vin_lookup_batch))
        .route("/decode/:vin", get(vin_lookup))
        .with_state(Arc::new(pool));

    info!("Binding to port 8080 and listening");
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    serve(listener, app).await.unwrap();

    Ok(())
}
