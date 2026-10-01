use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RRDDump {
    pub version: String,
    pub step: u32,
    pub lastupdate: u64,
    #[serde(rename = "ds")]
    pub data_sources: Vec<DataSource>,
    #[serde(rename = "rra")]
    pub archives: Vec<RRA>,
}
pub use super::types::{RRDFloat, RRDSample};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DataSource {
    pub name: String,
    #[serde(rename = "type")]
    pub ds_type: String,
    pub minimal_heartbeat: RRDFloat,
    pub min: RRDFloat,
    pub max: RRDFloat,
    pub last_ds: String,
    pub value: RRDFloat,
    pub unknown_sec: RRDFloat,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RRA {
    pub cf: String,
    pub pdp_per_row: u32,
    pub params: RRAParams,
    pub cdp_prep: CDPPrep,
    pub database: Database,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RRAParams {
    pub xff: RRDFloat,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CDPPrep {
    #[serde(rename = "ds")]
    pub ds: Vec<CDPDSStatus>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CDPDSStatus {
    pub primary_value: RRDFloat,
    pub secondary_value: RRDFloat,
    pub value: RRDFloat,
    pub unknown_datapoints: RRDFloat,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    #[serde(rename = "row")]
    pub rows: Vec<Row>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Row {
    #[serde(rename = "v")]
    pub values: RRDSample,
}

impl Default for RRA {
    fn default() -> Self {
        RRA {
            cf: String::new(),
            pdp_per_row: 0,
            params: RRAParams { xff: 0.0 },
            cdp_prep: CDPPrep { ds: Vec::new() },
            database: Database { rows: Vec::new() },
        }
    }
}