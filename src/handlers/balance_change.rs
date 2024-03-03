use crate::models::balance_change::BalanceChange;
use crate::utils::consts::{BALANCE_CHANGES_TABLE, KEYSPACE};
use crate::utils::errors::{map_error_to_status_code, DataApiError};
use crate::utils::params::DataApiQueryParams;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use scylla::Session;
use std::sync::Arc;

pub async fn get_account_balance_changes_handler(
    State(state): State<Arc<AppState>>,
    Path(account): Path<String>,
    params: axum::extract::Query<DataApiQueryParams>,
) -> anyhow::Result<Json<Vec<BalanceChange>>, StatusCode> {
    let match_value = format!("'{}'", account);
    // let limit = params.0.;
    match get_balance_changes(&state.scylla_session, "account", &match_value.to_string()).await {
        Ok(account) => Ok(Json(account)),
        Err(err) => {
            eprintln!("{}", err);
            Err(map_error_to_status_code(&err))
        }
    }
}

async fn get_balance_changes(
    session: &Session,
    field: &str,
    value: &str,
) -> Result<Vec<BalanceChange>, DataApiError> {
    let query = format!(
        "SELECT ledger_index, \
            tx_index, \
            node_index, \
            account, \
            change, \
            change_type, \
            counterparty, \
            currency, \
            final_balance, \
            timestamp, \
            tx_hash \
        from \"{}\".{} WHERE {}={};",
        KEYSPACE, BALANCE_CHANGES_TABLE, field, value
    );
    let mut query = scylla::query::Query::new(query);
    query.set_page_size(100);

    println!("Query: {}", query.contents);
    let query_result = session.query_paged(query, &[], None).await?;
    let balance_changes_iter = query_result.rows_typed_or_empty();

    // todo: better row error handling
    let balance_changes = balance_changes_iter
        .filter_map(|row| row.ok())
        .collect::<Vec<BalanceChange>>();

    if balance_changes.is_empty() {
        return Err(DataApiError::NoDataReturned);
    }
    println!("Returning {} balance changes", balance_changes.len());

    Ok(balance_changes)
}
