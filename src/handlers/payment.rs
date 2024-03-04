use crate::models::payment::Payment;
use crate::utils::consts::{KEYSPACE, PAYMENTS_TABLE};
use crate::utils::errors::{map_error_to_status_code, DataApiError};
use crate::utils::params::DataApiQueryParams;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use scylla::Session;
use std::sync::Arc;

pub async fn get_account_payments_handler(
    State(state): State<Arc<AppState>>,
    Path(account): Path<String>,
    params: axum::extract::Query<DataApiQueryParams>,
) -> anyhow::Result<Json<Vec<Payment>>, StatusCode> {
    println!("Params: {:?}", params.limit);

    let match_value = format!("'{}'", account);
    // let limit = params.0.;
    match get_payments(&state.scylla_session, "source", &match_value.to_string()).await {
        Ok(payments) => Ok(Json(payments)),
        Err(err) => {
            eprintln!("{}", err);
            Err(map_error_to_status_code(&err))
        }
    }
}

async fn get_payments(
    session: &Session,
    field: &str,
    value: &str,
) -> Result<Vec<Payment>, DataApiError> {
    let query = format!(
        "SELECT tx_hash, \
            ledger_index, \
            tx_index, \
            source, \
            source_currency, \
            source_currency_issuer, \
            destination, \
            destination_currency, \
            destination_currency_issuer, \
            amount, \
            delivered_amount, \
            transaction_cost, \
            destination_tag, \
            source_tag, \
            timestamp \
        from {} WHERE {}={};",
        PAYMENTS_TABLE, field, value
    );
    let mut query = scylla::query::Query::new(query);
    query.set_page_size(100);

    println!("Query: {}", query.contents);
    let query_result = session.query_paged(query, (), None).await?;
    let payments_iter = query_result.rows_typed_or_empty();

    // todo: better row error handling
    let payments = payments_iter
        .filter_map(|row| row.ok())
        .collect::<Vec<Payment>>();

    if payments.is_empty() {
        return Err(DataApiError::NoDataReturned);
    }
    println!("Returning {} balance changes", payments.len());

    Ok(payments)
}
