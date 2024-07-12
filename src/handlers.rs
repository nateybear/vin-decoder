use axum::extract::{Path, State};
use futures::stream::{FuturesOrdered, StreamExt};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{instrument, warn};

use crate::{sp_vin_decode, types::*, error::AppError};

#[instrument(skip_all)]
pub(crate) async fn vin_lookup(
    State(pool): State<Arc<PgPool>>,
    Path(vin): Path<String>,
) -> Result<String, AppError> {
    let mut conn = pool.acquire().await?;
    let vin: &str = &vin;

    Ok(serde_json::to_string(
        &sp_vin_decode(vin.into(), &mut conn).await?,
    )?)
}

#[instrument(skip_all)]
pub(crate) async fn vin_lookup_batch(
    State(pool): State<Arc<PgPool>>,
    body: String,
) -> Result<String, AppError> {
    let vins: Vec<_> = body.lines().collect();

    let mut query_out = FuturesOrdered::new();
    for vin in vins.clone() {
        let pool = pool.clone();
        query_out.push_back(async move {
            let mut conn = pool.acquire().await?;
            Ok(sp_vin_decode(vin.into(), &mut conn).await?)
        });
    }

    let query_out = query_out.collect::<Vec<Result<_, AppError>>>().await;

    let out = query_out.into_iter().zip(vins);
    let mut errors = vec![];

    let successes = out
        .filter_map(|(query_out, vin)| {
            if let Err(e) = &query_out {
                warn!(?vin, ?e, "Failed to decode vin");
                errors.push((vin.to_string(), e.to_string()));
                return None;
            }
            query_out.ok().map(|output| DecodingBatchOutput {
                vin: vin.to_string(),
                output,
            })
        })
        .collect::<Vec<DecodingBatchOutput>>();

    let results = DecodingBatchResults { successes, errors };

    Ok(serde_json::to_string(&results)?)
}
