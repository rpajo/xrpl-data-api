use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DataApiQueryParams {
    pub limit: Option<usize>,
    pub ledger_index: Option<u32>,
}
