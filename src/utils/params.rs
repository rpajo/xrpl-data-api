use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    limit: usize
}