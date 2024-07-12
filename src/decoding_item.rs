use chrono::NaiveDateTime;
use optfield::optfield;
use serde::Serialize;

#[optfield(pub(crate) DecodingItem, attrs, merge_fn)]
#[derive(Clone, Debug, Default)]
struct DecodingItemRaw {
    pub decoding_id: i32,
    pub source: String,
    pub created_on: NaiveDateTime,
    pub priority: i32,
    pub pattern_id: i32,
    pub keys: String,
    pub vin_schema_id: i32,
    pub wmi_id: i32,
    pub element_id: i32,
    pub attribute_id: i32,
    pub value: String,
}

#[optfield(pub(crate) DecodingOutput, attrs, merge_fn)]
#[derive(Clone, Debug, Serialize, Default)]
struct DecodingOutputRaw {
    pub group_name: String,
    pub variable: String,
    pub value: String,
    pub pattern_id: i32,
    pub vin_schema_id: i32,
    pub keys: String,
    pub element_id: i32,
    pub attribute_id: i32,
    pub created_on: NaiveDateTime,
    pub wmi_id: i32,
    pub code: String,
    pub data_type: String,
    pub decode: String,
    pub source: String,
}
