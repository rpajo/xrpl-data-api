use axum::http::StatusCode;
use scylla::transport::errors::QueryError;
use scylla::transport::query_result::SingleRowTypedError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataApiError {
    #[error("row type error ({0})")]
    RowTypedCast(#[from] SingleRowTypedError),

    #[error("querying data failed ({0})")]
    QueryFailed(#[from] QueryError),

    #[error("no data returned from query")]
    NoDataReturned,
}

pub fn map_error_to_status_code(err: &DataApiError) -> StatusCode {
    match err {
        DataApiError::RowTypedCast(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DataApiError::QueryFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DataApiError::NoDataReturned => StatusCode::NOT_FOUND,
    }
}
