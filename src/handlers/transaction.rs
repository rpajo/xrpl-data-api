use crate::models::transaction::Transaction;
use crate::utils::consts::{TRANSACTIONS_ACCOUNT_MV_TABLE, TRANSACTIONS_TABLE};
use crate::utils::errors::{map_error_to_status_code, DataApiError};
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use scylla::_macro_internal::SerializeRow;
use scylla::query::Query;
use scylla::Session;
use std::sync::Arc;

pub async fn get_transaction_by_hash(
    State(state): State<Arc<AppState>>,
    Path(tx_hash): Path<String>,
) -> anyhow::Result<Json<Transaction>, StatusCode> {
    match get_transactions(&state.scylla_session, "hash", (&tx_hash, )).await {
        Ok(mut transactions) => Ok(Json(transactions.pop().unwrap())),
        Err(err) => {
            eprintln!("{}", err);
            Err(map_error_to_status_code(&err))
        }
    }
}

pub async fn get_transaction_by_account(
    State(state): State<Arc<AppState>>,
    Path(account): Path<String>,
) -> anyhow::Result<Json<Vec<Transaction>>, StatusCode> {
    match get_transactions(&state.scylla_session, "account", (&account, )).await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(err) => {
            eprintln!("{}", err);
            Err(map_error_to_status_code(&err))
        }
    }
}

pub async fn get_transaction_by_ledger_index(
    State(state): State<Arc<AppState>>,
    Path(ledger_index): Path<i64>,
) -> anyhow::Result<Json<Vec<Transaction>>, StatusCode> {
    match get_transactions(&state.scylla_session, "ledger_index", (ledger_index, )).await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(err) => {
            eprintln!("{}", err);
            Err(map_error_to_status_code(&err))
        }
    }
}

async fn get_transactions(
    session: &Session,
    field: &str,
    values: impl SerializeRow,
) -> Result<Vec<Transaction>, DataApiError> {
    let table = if field == "account" {
        TRANSACTIONS_ACCOUNT_MV_TABLE
    } else {
        TRANSACTIONS_TABLE
    };
    // ! NOTE: select column order is important. Must match the order of the struct fields
    let query = format!(
        "SELECT account, \
            hash, \
            ctid, \
            ledger_index, \
            tx_index, \
            tx_type, \
            timestamp, \
            flags, \
            fee, \
            sequence, \
            result, \
            meta, \
            tx \
        from {} WHERE {}=?;",
        table, field
    );
    let mut query: Query = Query::new(query);
    query.set_page_size(100);

    println!("Query: {}", query.contents);
    let query_result = session.query_paged(query, values, None).await?;
    let transactions_iter = query_result.rows_typed_or_empty();

    // todo: better row error handling
    let transactions = transactions_iter
        .filter_map(|row| row.ok())
        .collect::<Vec<Transaction>>();

    if transactions.is_empty() {
        return Err(DataApiError::NoDataReturned);
    }
    println!("Found {} transactions", transactions.len());

    Ok(transactions)
}
