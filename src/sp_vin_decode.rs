use crate::vin_descriptor;
use anyhow::Result;
use sqlx::{query, PgConnection};

pub struct SpVinDecodeArgs<'a> {
    v: &'a str,
    include_private: Option<bool>,
    year: Option<u8>,
    include_all: Option<bool>,
    no_output: Option<bool>,
}

pub async fn sp_vin_decode(args: SpVinDecodeArgs<'_>, conn: &mut PgConnection) -> Result<()> {
    let vin = args.v.trim().to_uppercase();
    let descriptor = vin_descriptor(&vin);
    let dmy = query!(
        "select model_year from vin_descriptor where descriptor = $1",
        descriptor
    )
    .fetch_optional(conn)
    .await?
    .map(|x| x.model_year)
    .flatten();

    Ok(())
}
