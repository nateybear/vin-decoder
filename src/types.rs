use chrono::NaiveDateTime;
use optfield::optfield;
use serde::{Deserialize, Serialize};

/*


    Decoding Output Types

*/
#[optfield(pub(crate) DecodingItem, attrs, merge_fn)]
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
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

#[derive(Serialize, Default, Debug)]
pub(crate) struct DecodingBatchOutput {
    pub vin: String,
    #[serde(flatten)]
    pub output: Vec<DecodingOutput>,
}

#[derive(Serialize, Debug)]
pub(crate) struct DecodingBatchResults {
    pub(crate) successes: Vec<DecodingBatchOutput>,
    pub(crate) errors: Vec<(String, String)>,
}

/*

        ARGUMENT TYPES

*/
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SpVinDecodeArgs<'a> {
    pub v: &'a str,
    pub include_private: Option<bool>,
    pub year: Option<u8>,
    pub include_all: Option<bool>,
    pub no_output: Option<bool>,
}

impl<'a> From<&'a str> for SpVinDecodeArgs<'a> {
    fn from(v: &'a str) -> Self {
        Self {
            v,
            include_private: None,
            year: None,
            include_all: None,
            no_output: None,
        }
    }
}

pub struct SpVinDecodeCoreArgs<'a> {
    pub pass: i32,
    pub vin: &'a str,
    pub model_year: Option<i32>,
    pub model_year_source: Option<&'a str>,
    pub include_private: Option<bool>,
    pub include_not_publicly_available: Option<bool>,
}
