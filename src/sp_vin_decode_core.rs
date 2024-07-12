use crate::vin_wmi::vin_wmi;
use crate::DecodingItem;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use regex::Regex;
use sqlx::{query, query_as, PgConnection};

pub struct SpVinDecodeCoreArgs<'a> {
    pub pass: i32,
    pub vin: &'a str,
    pub model_year: Option<i32>,
    pub model_year_source: Option<&'a str>,
    pub include_private: Option<bool>,
    pub include_not_publicly_available: Option<bool>,
}

pub async fn sp_vin_decode_core(
    SpVinDecodeCoreArgs {
        pass,
        vin,
        model_year,
        model_year_source,
        include_private,
        include_not_publicly_available,
    }: SpVinDecodeCoreArgs<'_>,
    conn: &mut PgConnection,
) -> Result<Vec<DecodingItem>> {
    tracing::debug!("model year: {:?}", model_year);
    let wmi = vin_wmi(vin);
    tracing::debug!("wmi: {:?}", wmi);
    let keys = match vin {
        v if v.len() > 9 => v[3..8].to_string() + "|" + &v[9..17],
        v if v.len() > 3 => v[3..8].to_string(),
        _ => "".to_string(),
    };
    tracing::debug!("keys: {:?}", keys);

    let wmi_id = query!(
        "select id from wmi where wmi = $1 and ($2 or public_availability_date <= current_timestamp)", 
        wmi,
        include_not_publicly_available
    )
        .fetch_optional(&mut *conn)
        .await?
        .and_then(|x| x.id)
        .ok_or(anyhow!("Could not decode manufacturer from WMI index"))?;

    let mut decodings = Vec::<DecodingItem>::new();

    macro_rules! insert_decodings {
        ($query: expr, $($args: tt)*) => {
            query_as!(DecodingItem, $query, $($args)*)
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .for_each(|x| decodings.push(x));
        };
    }

    insert_decodings!(
        "select
            $1::integer as decoding_id,
            'Pattern' as source,
            coalesce(p.updated_on, p.created_on) as created_on,
            wvs.year_from as priority,
            p.id as pattern_id,
            upper(p.keys) as keys,
            p.vin_schema_id,
            wvs.wmi_id,
            p.element_id,
            p.attribute_id,
            'XXX' as value
        from
            pattern p
            join element e on p.element_id = e.id
            join vin_schema vs on p.vin_schema_id = vs.id
            join wmi__vin_schema wvs on 
                vs.id = wvs.vin_schema_id
                and ($6::integer is null or $6 between wvs.year_from and coalesce(wvs.year_to, 2999))
            join wmi w on wvs.wmi_id = w.id and w.wmi = $2
        where
            $3 similar to concat(replace(p.keys, '*', '_'), '%')
            and not p.element_id in (26, 27, 29, 39)
            and e.decode is not null
            and (coalesce(e.is_private, 0) = 0 or $4)
            and ($5 or w.public_availability_date <= current_timestamp)
            and ($5 or coalesce(vs.tobe_q_ced, true))",
        pass,
        wmi,
        keys,
        include_private,
        include_not_publicly_available,
        model_year
    );

    /*
     *
     *   ENGINE INFO, IF APPLICABLE
     *
     */

    if let Some(item) = decodings
        .iter()
        .filter(|x| x.element_id == Some(18) && x.decoding_id == Some(pass))
        .sorted_by(|x, y| x.priority.unwrap_or(-1).cmp(&y.priority.unwrap_or(-1)))
        .next()
    {
        insert_decodings!(
            "select
                $1::integer as decoding_id,
                'EngineModelPattern' as source,
                coalesce(p.updated_on, p.created_on) as created_on,
                50 as priority,
                $2::integer as pattern_id,
                $3 as keys,
                $4::integer as vin_schema_id,
                $5::integer as wmi_id,
                p.element_id,
                p.attribute_id,
                'XXX' as value
            from
                engine_model em
                join engine_model_pattern p on em.id = p.engine_model_id
                join element e on p.element_id = e.id
            where
                em.name = $6",
            pass,
            item.pattern_id,
            item.keys,
            item.vin_schema_id,
            wmi_id,
            item.attribute_id.map(|x| x.to_string())
        );
    }

    /*
     *
     *   VEHICLE TYPE
     *
     */
    insert_decodings!(
        "select
            $1::integer as decoding_id,
            'VehType' as source,
            coalesce(w.updated_on, w.created_on) as created_on,
            100 as priority,
            NULL::integer as pattern_id,
            upper($2) as keys,
            NULL::integer as vin_schema_id,
            w.id as wmi_id,
            39 as element_id,
            t.id as attribute_id,
            upper(t.name) as value
        from
            wmi w
            join vehicle_type t on t.id = w.vehicle_type_id
        where
            wmi = $2
            and ($3 or w.public_availability_date <= current_timestamp)",
        pass,
        wmi,
        include_not_publicly_available
    );

    /*
     *
     *    MANUFACTURER INFORMATION
     *
     */

    let (mfr_id, mfr_name) = query!(
        "select
            t.id,
            upper(t.name) as name
        from
            wmi w
            join manufacturer t on t.id = w.manufacturer_id
        where
            wmi = $1
            and ($2 or w.public_availability_date <= current_timestamp)",
        wmi,
        include_not_publicly_available
    )
    .fetch_optional(&mut *conn)
    .await?
    .map_or((None, None), |x| (x.id, x.name));

    let mfr_item = DecodingItem {
        decoding_id: Some(pass),
        source: Some("Manuf. Name".to_string()),
        priority: Some(100),
        created_on: None,
        pattern_id: None,
        keys: Some(wmi.to_uppercase()),
        vin_schema_id: None,
        wmi_id: Some(wmi_id),
        element_id: Some(27),
        attribute_id: mfr_id,
        value: mfr_name,
    };

    decodings.push(mfr_item.clone());

    decodings.push(DecodingItem {
        source: Some("Manuf. Id".to_string()),
        value: mfr_id.map(|x| x.to_string()),
        element_id: Some(157),
        ..mfr_item
    });

    if let Some(model_year) = model_year {
        decodings.push(DecodingItem {
            decoding_id: Some(pass),
            source: Some("ModelYear".to_string()),
            created_on: None,
            priority: Some(100),
            pattern_id: None,
            keys: model_year_source.map(|x| x.to_owned()),
            vin_schema_id: None,
            wmi_id: None,
            element_id: Some(29),
            attribute_id: Some(model_year),
            value: Some(model_year.to_string()),
        });
    }

    let formula_keys = Regex::new(r"\d")?.replace_all(&keys, "#").to_string();
    insert_decodings!(
        "select
            $1::integer as decoding_id,
            'Formula Pattern' as source,
            coalesce(p.updated_on, p.created_on) as created_on,
            100::integer as priority,
            p.id as pattern_id,
            NULL as keys,
            p.vin_schema_id,
            NULL::integer as wmi_id,
            p.element_id,
            p.attribute_id,
            substring($2, position('#' in p.keys), ((length(p.keys) - position('#' in reverse(p.keys)) + 1) - (position('#' in p.keys)) + 1))::text as value
        from
            pattern p
            join element e on p.element_id = e.id
        where
            p.vin_schema_id in
            ( select wvs.vin_schema_id
                from 
                    wmi w
                    join wmi__vin_schema wvs 
                        on w.id = wvs.wmi_id
                        and ($3::integer is null or $3 between wvs.year_from and coalesce(wvs.year_to, 2999))
                where
                    w.wmi = $4 and ($3 is null or $3 between wvs.year_from and coalesce(wvs.year_to, 2999))
                    and ($5 or w.public_availability_date <= current_timestamp)
            )
            and position('#' in p.keys) > 0
            and p.element_id not in (26, 27, 29, 39)
            and $6 similar to concat(replace(p.keys, '*', '_'), '%')",
        pass,
        keys,
        model_year,
        wmi,
        include_not_publicly_available,
        &formula_keys
    );

    /*
     *
     *   REMOVE SOME ITEMS?
     *
     */

    let (mut to_filter, mut to_keep): (Vec<DecodingItem>, Vec<DecodingItem>) =
        decodings.into_iter().partition(|d| {
            d.decoding_id == Some(pass)
                && ![121, 129, 150, 154, 155, 114, 169, 186].contains(&d.element_id.unwrap_or(0))
        });

    to_filter = to_filter
        .into_iter()
        .into_grouping_map_by(|d| d.element_id)
        .max_by_key(|_key, d| {
            (
                d.priority,
                d.created_on,
                -TryInto::<i8>::try_into(
                    d.keys
                        .clone()
                        .unwrap_or("".to_owned())
                        .replace('*', "")
                        .len(),
                )
                .unwrap_or(100),
            )
        })
        .into_values()
        .collect();

    to_keep.extend(to_filter);

    decodings = to_keep;
    /*
     *
     *   DECODE MODEL INFO IF AVAILABLE
     *
     */

    let model_id = decodings
        .iter()
        .find(|d| d.decoding_id == Some(pass) && d.element_id == Some(28))
        .cloned();

    if let Some(ref mid) = model_id {
        query!(
            "select
                mk.id as attribute_id,
                upper(mk.name) as value
            from
                make__model mm
                join make mk on mm.make_id = mk.id
            where
                mm.model_id = $1",
            mid.attribute_id
        )
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .for_each(|mm| {
            let mid = mid.clone();
            decodings.push(DecodingItem {
                decoding_id: Some(pass),
                source: Some("pattern - model".to_string()),
                created_on: None,
                priority: Some(1000),
                pattern_id: mid.pattern_id,
                keys: mid.keys,
                vin_schema_id: mid.vin_schema_id,
                wmi_id: None,
                element_id: Some(26),
                attribute_id: mm.attribute_id,
                value: mm.value,
            });
        });
    } else {
        let matched_make = query!(
            "select
                coalesce(w.updated_on, w.created_on) as created_on,
                w.id as wmi_id,
                t.id as attribute_id,
                upper(t.name) as value
            from
                wmi w
                join wmi__make wm on wm.wmi_id = w.id
                join make t on t.id = wm.make_id
            where
                wmi = $1
                and ($2 or w.public_availability_date <= current_timestamp)",
            wmi,
            include_not_publicly_available
        )
        .fetch_all(&mut *conn)
        .await?;

        if let Ok(Some(mm)) = matched_make.into_iter().at_most_one() {
            decodings.push(DecodingItem {
                decoding_id: Some(pass),
                source: Some("Make".to_string()),
                created_on: mm.created_on,
                priority: Some(-100),
                pattern_id: None,
                keys: wmi.clone().into(),
                vin_schema_id: None,
                wmi_id: mm.wmi_id,
                element_id: Some(26),
                attribute_id: mm.attribute_id,
                value: mm.value,
            });
        }
    }

    /*
     *
     *   UNIT CONVERSIONS
     *   skipped, lines 250-295 of spVinDecode_Core.txt
     *
     */

    /*
     *
     *
     *   MATCHING VEHICLE SPEC PATTERNS
     *
     */
    let vehicle_type = decodings
        .iter()
        .filter(|x| x.decoding_id == Some(pass) && x.element_id == Some(39))
        .next()
        .map(|x| x.attribute_id)
        .flatten();

    if let Some(vt) = vehicle_type {
        tracing::debug!("vt: {:?}", vt);
        let mut temp_patterns = query!(
            "select distinct sp.id, s.tobe_q_ced
         from vehicle_spec_schema s
            join v_spec_schema_pattern sp on s.id = sp.schema_id
            join vehicle_spec_pattern p on sp.id = p.v_spec_schema_pattern_id
            join vehicle_spec_schema__model vssm on vssm.vehicle_spec_schema_id = s.id
            left join vehicle_spec_schema__year vssy on vssy.vehicle_spec_schema_id = s.id
            join wmi__make wm on wm.make_id = s.make_id
            join wmi on wmi.id = wm.wmi_id
            where
                wmi.wmi = $1
                and s.vehicle_type_id = $2
                and vssm.model_id = $3
                and (vssy.year = $4 or vssy.id is null)
                and p.is_key = '1'
                and ($5 or (coalesce(s.tobe_q_ced, true)))",
            wmi,
            vt,
            model_id.map(|m| m.attribute_id).flatten(),
            model_year,
            include_not_publicly_available
        )
        .fetch_all(&mut *conn)
        .await?;

        let current_decodings = decodings
            .iter()
            .filter(|d| d.decoding_id == Some(pass))
            .enumerate();

        // weird query in lines 322-332, remove any patterns that match multiple decodings?
        let mut temp_patterns_exclude = query!(
            "select p.v_spec_schema_pattern_id, p.element_id, p.attribute_id
            from vehicle_spec_pattern p
            where
                p.v_spec_schema_pattern_id = any($1)
                and p.is_key = '1'",
            &temp_patterns
                .iter()
                .filter_map(|x| x.id)
                .collect::<Vec<_>>()
        )
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .flat_map(|patt| {
            let element_id = patt.element_id;
            let attribute_id = patt.attribute_id;
            let v_spec_schema_pattern_id = patt.v_spec_schema_pattern_id;
            current_decodings
                .clone()
                .filter(move |(_, x)| x.element_id == element_id && x.attribute_id == attribute_id)
                .map(move |(i, x)| (i, x, v_spec_schema_pattern_id))
        })
        .into_group_map_by(|(_, _, vid)| *vid)
        .into_iter()
        .filter(|(_vid, ds)| ds.iter().map(|x| x.0).dedup().try_len().unwrap_or(0) != ds.len())
        .map(|(vid, _)| vid);

        temp_patterns = temp_patterns
            .into_iter()
            .filter(|x| temp_patterns_exclude.contains(&x.id))
            .collect::<Vec<_>>();

        let relevant_decodings = decodings
            .iter()
            .filter_map(|d| match d {
                x if [1, 114, 121, 129, 150, 154, 155, 169, 186]
                    .contains(&x.element_id.unwrap_or(0))
                    && x.decoding_id == Some(pass) =>
                {
                    Some(x.element_id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        query!(
            "select distinct
                vsp.is_key,
                vsvp.schema_id,
                vsp.v_spec_schema_pattern_id,
                vsp.element_id,
                vsp.attribute_id,
                vsvp.id,
                coalesce(vsp.updated_on, vsp.created_on) as changed_on
            from vehicle_spec_pattern vsp
                join v_spec_schema_pattern vsvp on vsvp.id = vsp.v_spec_schema_pattern_id
            where
                vsp.is_key = '0'"
        )
        .fetch_all(&mut *conn)
        .await?
        .iter()
        .flat_map(|x| {
            temp_patterns
                .iter()
                .filter(|y| x.id == y.id)
                .map(move |y| (x, y))
        })
        .filter(|(x, _y)| !relevant_decodings.contains(&x.element_id))
        .for_each(|(x, _)| {
            decodings.push(DecodingItem {
                decoding_id: Some(pass),
                source: Some("Vehicle Specs".into()),
                created_on: x.changed_on,
                priority: Some(-100),
                pattern_id: x.v_spec_schema_pattern_id,
                keys: Some("".into()),
                vin_schema_id: x.schema_id,
                wmi_id: None,
                element_id: x.element_id,
                attribute_id: x.attribute_id,
                value: Some("XXX".into()),
            });
        });
    }

    /*
     *
     *
     *   VEHICLE TYPE
     *
     */

    let vehicle_type = decodings
        .iter()
        .find(|d| d.decoding_id == Some(pass) && d.element_id == Some(39))
        .map(|d| d.attribute_id);

    if let Some(vt) = vehicle_type {
        let relevent_decodings = decodings
            .iter()
            .filter_map(|d| {
                if d.decoding_id == Some(pass) {
                    d.element_id
                } else {
                    None
                }
            })
            .dedup()
            .collect::<Vec<_>>();

        query!(
            "select
                coalesce(dv.updated_on, dv.created_on) as created_on,
                dv.element_id,
                dv.default_value,
                case when e.data_type = 'lookup' and dv.default_value = '0' then 'Not Applicable' else 'XXX' end as value,
                dv.vehicle_type_id
            from
                default_value dv
                join element e on dv.element_id = e.id")
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .filter(|r| {
                r.vehicle_type_id == vt && r.default_value.is_some() && !relevent_decodings.contains(&r.element_id.unwrap_or(-999))
            })
            .for_each(|r| {
                decodings.push(DecodingItem {
                    decoding_id: Some(pass),
                    source: Some("default".into()),
                    created_on: r.created_on,
                    priority: Some(10),
                    pattern_id: None,
                    keys: None,
                    vin_schema_id: None,
                    wmi_id: None,
                    element_id: r.element_id,
                    attribute_id: r.default_value,
                    value: r.value,
                });
            });
    }

    Ok(decodings)
}
