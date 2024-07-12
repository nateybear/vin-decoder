use anyhow::Result;
use sqlx::query;
use sqlx::PgConnection;
use sqlx::Row;

pub async fn element_attribute_value(
    element_id: i32,
    attribute_id: i32,
    conn: &mut PgConnection,
) -> Result<Option<String>> {
    macro_rules! table {
        ($tab: tt) => {
            query(&format!("select name from {} where id = $1", $tab))
                .bind(attribute_id)
                .fetch_optional(&mut *conn)
                .await?
                .map(|x| x.get(0))
        };
    }

    let out = match element_id {
        2 => table!("battery_type"),
        3 => table!("bed_type"),
        4 => table!("body_cab"),
        5 => table!("body_style"),
        10 => table!("destination_market"),
        15 => table!("drive_type"),
        23 => table!("entertainment_system"),
        24 => table!("fuel_type"),
        25 => table!("gross_vehicle_weight_rating"),
        26 => table!("make"),
        27 => table!("manufacturer"),
        28 => table!("model"),
        36 => table!("steering"),
        37 => table!("transmission"),
        39 => table!("vehicle_type"),
        42 => table!("brake_system"),
        55 => table!("air_bag_locations"),
        56 => table!("air_bag_locations"),
        60 => table!("wheel_base_type"),
        62 => table!("valvetrain_design"),
        64 => table!("engine_configuration"),
        65 => table!("air_bag_loc_front"),
        66 => table!("fuel_type"),
        67 => table!("fuel_delivery_type"),
        69 => table!("air_bag_loc_knee"),
        72 => table!("evdrive_unit"),
        75 => table!("country"),
        78 => table!("pretensioner"),
        79 => table!("seat_belts_all"),
        81 => table!("adaptive_cruise_control"),
        86 => table!("abs"),
        87 => table!("auto_brake"),
        88 => table!("blind_spot_monitoring"),
        96 => table!("v_ncsabody_type"),
        97 => table!("v_ncsamake"),
        98 => table!("v_ncsamodel"),
        99 => table!("ecs"),
        100 => table!("traction_control"),
        101 => table!("forward_collision_warning"),
        102 => table!("lane_departure_warning"),
        103 => table!("lane_keep_system"),
        104 => table!("rear_visibility_camera"),
        105 => table!("park_assist"),
        107 => table!("air_bag_locations"),
        116 => table!("trailer_type"),
        117 => table!("trailer_body_type"),
        122 => table!("cooling_type"),
        126 => table!("electrification_level"),
        127 => table!("charger_level"),
        135 => table!("turbo"),
        143 => table!("error_code"),
        145 => table!("axle_configuration"),
        148 => table!("bus_floor_config_type"),
        149 => table!("bus_type"),
        151 => table!("custom_motorcycle_type"),
        152 => table!("motorcycle_suspension_type"),
        153 => table!("motorcycle_chassis_type"),
        168 => table!("tpms"),
        170 => table!("dynamic_brake_support"),
        171 => table!("pedestrian_automatic_emergency_braking"),
        172 => table!("auto_reverse_system"),
        173 => table!("automatic_pedestrain_alerting_sound"),
        174 => table!("can__aacn"),
        175 => table!("edr"),
        176 => table!("keyless_ignition"),
        177 => table!("daytime_running_light"),
        178 => table!("lower_beam_headlamp_light_source"),
        179 => table!("semiautomatic_headlamp_beam_switching"),
        180 => table!("adaptive_driving_beam"),
        183 => table!("rear_cross_traffic_alert"),
        184 => table!("gross_vehicle_weight_rating"),
        185 => table!("gross_vehicle_weight_rating"),
        190 => table!("gross_vehicle_weight_rating"),
        192 => table!("rear_automatic_emergency_braking"),
        193 => table!("blind_spot_intervention"),
        194 => table!("lane_centering_assistance"),
        195 => table!("non_land_use"),
        _ => Some(attribute_id.to_string()),
    };

    Ok(out)
}
