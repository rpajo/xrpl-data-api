use std::sync::Arc;
use crate::AppState;

use axum::http::StatusCode;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use scylla::{Session};
use crate::utils::consts::{DAILY_LEDGERS_TABLE, KEYSPACE, LEDGER_TABLE};
use crate::models::ledger::LedgerScylla;
use crate::utils::errors::{DataApiError, map_error_to_status_code};

pub async fn get_ledger_handler(
    State(state): State<Arc<AppState>>,
    Path(ledger_identifier): Path<String>,
) -> Result<Json<LedgerScylla>, StatusCode> {
    let time_parsed = ledger_identifier.parse::<DateTime<Utc>>();
    if let Ok(close_time) = time_parsed {
        println!("Finding ledger with close time: {}", close_time.format("%Y-%m-%d"));
        match get_ledger_at_time(&state.scylla_session, close_time).await {
            Ok(ledger) => Ok(Json(ledger)),
            Err(err) => {
                eprintln!("{}", err);
                Err(map_error_to_status_code(&err))
            }
        }
    } else {
        let search_terms = match ledger_identifier.parse::<u32>() {
            Ok(ledger_index) => (ledger_index.to_string(), "ledger_index"),
            Err(_) => (format!("'{}'", ledger_identifier), "ledger_hash")
        };
        println!("Finding ledger with {}", search_terms.1);

        match get_ledger(&state.scylla_session, search_terms.1, &search_terms.0).await {
            Ok(ledger) => Ok(Json(ledger)),
            Err(err) => {
                eprintln!("{}", err);
                Err(map_error_to_status_code(&err))
            }
        }
    }
}

pub async fn get_ledger_at_time(session: &Session, close_time: DateTime<Utc>) -> Result<LedgerScylla, DataApiError> {
    let unix_time_ms = close_time.timestamp_millis();
    let query = format!(
        "SELECT ledger_close_day, ledger_index, toUnixTimestamp(close_time) \
        from \"{}\".{} \
        WHERE ledger_close_day = toDate({})",
        KEYSPACE, DAILY_LEDGERS_TABLE, unix_time_ms
    );
    println!("Query: {}", query);
    let query_result = session.query(query, &[]).await?;
    if let Ok(num_of_rows) = query_result.rows_num() {
        if num_of_rows == 0 {
            println!("No ledgers found on day: {}", close_time.format("%Y-%m-%d"));
            return Err(DataApiError::NoDataReturned);
        }
    }
    let ledger_in_day = query_result.rows_typed_or_empty::<(NaiveDate, i64, i64)>();

    println!("Find ledger closest after {}", unix_time_ms);
    let last_closed_ledger = ledger_in_day
        .map(|l| l.expect("daly ledger should be valid"))
        .filter(|l| l.2 <= unix_time_ms)
        .map(|l| l.1)
        .max();

    match last_closed_ledger {
        Some(index) => {
            let ledger_result = get_ledger(
                session,
                "ledger_index",
                &index.to_string(),
            ).await?;
            Ok(ledger_result)
        }
        None => Err(DataApiError::NoDataReturned),
    }
}

async fn get_ledger(session: &Session, field: &str, value: &str) -> Result<LedgerScylla, DataApiError> {
    // ! NOTE: select column order is important. Must match the order of the struct fields
    let query = format!(
        "SELECT ledger_index, \
            ledger_hash, \
            parent_hash, \
            account_hash, \
            transaction_hash, \
            close_flags, \
            close_time, \
            parent_close_time, \
            total_coins, \
            tx_count, \
            ledger_processed \
            from \"{}\".{} WHERE {}={};",
        KEYSPACE, LEDGER_TABLE, field, value
    );
    println!("Query: {}", query);
    let query_result = session.query(query, &[]).await?;
    if let Ok(num_of_rows) = query_result.rows_num() {
        if num_of_rows == 0 {
            return Err(DataApiError::NoDataReturned);
        }
    }
    let ledger = query_result.single_row_typed::<LedgerScylla>()?;

    Ok(ledger)
}
