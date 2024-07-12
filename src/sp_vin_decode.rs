use crate::{
    element_attribute_value::element_attribute_value,
    sp_vin_decode_core::sp_vin_decode_core,
    types::{DecodingItem, DecodingOutput},
    types::{SpVinDecodeArgs, SpVinDecodeCoreArgs},
    vin_descriptor,
    vin_model_year::vin_model_year,
};
use anyhow::Result;
use chrono::{Datelike, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::{query, PgConnection};

lazy_static! {
    static ref MODEL_YEAR: i32 = Utc::now().year() + 1;
}

pub async fn sp_vin_decode(
    args: SpVinDecodeArgs<'_>,
    conn: &mut PgConnection,
) -> Result<Vec<DecodingOutput>> {
    let vin = args.v.trim().to_uppercase();
    let descriptor = vin_descriptor(&vin);
    tracing::debug!("Descriptor: {:?}", descriptor);
    let dmy = query!(
        "select model_year from vin_descriptor where descriptor = $1",
        descriptor
    )
    .fetch_optional(&mut *conn)
    .await?
    .into_iter()
    .filter_map(|x| match x.model_year {
        Some(y) if (1980..=*MODEL_YEAR).contains(&y) => Some(y),
        _ => None,
    })
    .next();

    let mut decodings = Vec::<DecodingItem>::new();

    if let Some(ref dmy) = dmy {
        let vin_decode_core_args = SpVinDecodeCoreArgs {
            vin: &vin,
            pass: 1,
            model_year: Some(*dmy),
            model_year_source: Some(&descriptor),
            include_private: args.include_private,
            include_not_publicly_available: Some(false),
        };

        decodings.extend(sp_vin_decode_core(vin_decode_core_args, &mut *conn).await?);
    } else {
        // skipping lines 53-67
        let rmy = vin_model_year(&vin);
        let vin_decode_core_args = SpVinDecodeCoreArgs {
            vin: &vin,
            pass: 3,
            model_year: rmy,
            model_year_source: Some(&descriptor),
            include_private: args.include_private,
            include_not_publicly_available: Some(false),
        };
        decodings.extend(sp_vin_decode_core(vin_decode_core_args, &mut *conn).await?);
    }

    for d in decodings.iter_mut() {
        if d.value == Some("XXX".into()) {
            if let (Some(e), Some(a)) = (d.element_id, d.attribute_id) {
                d.value = element_attribute_value(e, a, conn).await?;
            }
        }
    }

    let value_regex = Regex::new("((9)|(13)|(10))")?;

    let output = query!(
        "
        select
            group_name,
            name as variable,
            id as element_id,
            code,
            data_type,
            decode
        from 
            element
        where
            coalesce(decode, '') <> ''
            and ($1 or (coalesce($1, false) = false and id is not null))
            and ($2 or coalesce(is_private, 0) = 0)
        order by
            case coalesce(group_name, '')
                when '' then 0
                when 'General' then 1
		when 'Exterior / Body' then 2
		when 'Exterior / Dimension' then 3
		when 'Exterior / Truck' then 4
		when 'Exterior / Trailer' then 5
		when 'Exterior / Wheel tire' then 6
		when 'Interior' then 7
		when 'Interior / Seat' then 8
		when 'Mechanical / Transmission' then 9
		when 'Mechanical / Drivetrain' then 10
		when 'Mechanical / Brake' then 11
		when 'Mechanical / Battery' then 12
		when 'Mechanical / Battery / Charger' then 13
		when 'Engine' then 14
		when 'Passive Safety System' then 15
		when 'Passive Safety System / Air Bag Location' then 16
		when 'Active Safety System' then 17
		when 'Internal' then 18
		else 99 
            end,
            id",
        args.include_all,
        args.include_private
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|x| {
        let DecodingItem {
            source,
            created_on,
            pattern_id,
            keys,
            vin_schema_id,
            wmi_id,
            element_id,
            attribute_id,
            value,
            ..
        } = decodings
            .iter()
            .find(|d| d.element_id == x.element_id)
            .cloned()
            .unwrap_or(DecodingItem::default());

        DecodingOutput {
            group_name: x.group_name,
            variable: x.variable,
            value: value.map(|v| value_regex.replace_all(&v, " ").to_string()),
            pattern_id,
            vin_schema_id,
            keys,
            element_id,
            attribute_id,
            created_on,
            wmi_id,
            code: x.code,
            data_type: x.data_type,
            decode: x.decode,
            source,
        }
    })
    .collect::<_>();

    Ok(output)
}
