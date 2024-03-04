use crate::AppState;
use std::sync::Arc;

use crate::handlers::daily_ledger::get_last_closed_ledger;
use crate::models::ledger::Ledger;
use crate::utils::consts::{KEYSPACE, LEDGER_TABLE};
use crate::utils::errors::{map_error_to_status_code, DataApiError};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use scylla::Session;

pub async fn get_ledger_handler(
    State(state): State<Arc<AppState>>,
    Path(ledger_identifier): Path<String>,
) -> Result<Json<Ledger>, StatusCode> {
    let time_parsed = ledger_identifier.parse::<DateTime<Utc>>();
    if let Ok(close_time) = time_parsed {
        println!(
            "Finding ledger with close time: {}",
            close_time.format("%Y-%m-%d")
        );
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
            Err(_) => (format!("'{}'", ledger_identifier), "ledger_hash"),
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

pub async fn get_ledger_at_time(
    session: &Session,
    close_time: DateTime<Utc>,
) -> Result<Ledger, DataApiError> {
    let latest_closed_ledger = get_last_closed_ledger(session, close_time).await?;

    let ledger_result = get_ledger(
        session,
        "ledger_index",
        &latest_closed_ledger.ledger_index.to_string(),
    )
    .await?;
    Ok(ledger_result)
}

async fn get_ledger(session: &Session, field: &str, value: &str) -> Result<Ledger, DataApiError> {
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
    let ledger = query_result.single_row_typed::<Ledger>()?;

    Ok(ledger)
}
