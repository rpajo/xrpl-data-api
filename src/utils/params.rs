use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DataApiQueryParams {
    pub limit: Option<i32>,
    pub ledger_index: Option<u32>,
}
