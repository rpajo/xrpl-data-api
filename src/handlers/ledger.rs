use std::sync::Arc;
use crate::AppState;

use axum::http::StatusCode;
use anyhow::Result;
use axum::extract::{Path, State};
use axum::Json;
use chrono::NaiveDate;
use scylla::Session;
use crate::consts::{DAILY_LEDGERS_TABLE, KEYSPACE, LEDGER_TABLE};
use crate::models::ledger::LedgerScylla;

pub async fn ledger_by_index_handler(
    State(state): State<Arc<AppState>>,
    Path(ledger_index): Path<u32>
) -> Result<Json<LedgerScylla>, StatusCode> {
    match get_ledger(&state.scylla_session, "ledger_index", &ledger_index.to_string()).await {
        Ok(ledger) => Ok(Json(ledger)),
        Err(err) => {
            eprintln!("Error fetching ledger: {:?}", err);
            Err(StatusCode::NOT_FOUND)
        },
    }
}

pub async fn ledger_by_hash_handler(
    State(state): State<Arc<AppState>>,
    Path(ledger_hash): Path<String>
) -> Result<Json<LedgerScylla>, StatusCode> {
    let match_value = format!("'{}'", ledger_hash);
    match get_ledger(&state.scylla_session, "ledger_hash", &match_value).await {
        Ok(ledger) => Ok(Json(ledger)),
        Err(err) => {
            eprintln!("Error fetching ledger: {:?}", err);
            Err(StatusCode::NOT_FOUND)
        },
    }
}

pub async fn ledger_at_time_handler(
    State(state): State<Arc<AppState>>,
    Path(unix_time): Path<i64>) -> Result<Json<LedgerScylla>, StatusCode> {
    let time = unix_time * 1000;
    let query = format!(
        "SELECT ledger_close_day, ledger_index, toUnixTimestamp(close_time) \
        from \"{}\".{} \
        WHERE ledger_close_day = toDate({})",
        KEYSPACE, DAILY_LEDGERS_TABLE, time
    );
    println!("Query: {}", query);
    let query_result = state.scylla_session.query(query, &[]).await;
    let ledger_in_day = match query_result {
        Ok(result) => Some(result.rows_typed_or_empty::<(NaiveDate, i64, i64)>()),
        Err(err) => {
            eprintln!("{:?}", err);
            None
        }
    };

    if ledger_in_day.is_none() {
        eprintln!("Error getting ledger by close time");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    /* let last_closed_ledger = ledger_in_day
    .unwrap()
    .map(|l| l.unwrap())
    .filter(|l| l.2 < time)
    .map(|l| l.2)
    .max(); */

    // todo: order if out of order?
    let mut last_closed_ledger = None;
    for ledger in ledger_in_day.unwrap() {
        let ledger_data = ledger.unwrap();
        println!("{:?}", ledger_data);
        if ledger_data.2 < time {
            last_closed_ledger = Some(ledger_data.1);
            break;
        }
    }

    match last_closed_ledger {
        Some(index) => match get_ledger(
            &state.scylla_session, 
            "ledger_index", 
            &index.to_string()
        ).await {
            Ok(ledger) => Ok(Json(ledger)),
            Err(err) => {
                eprintln!("Error fetching ledger: {:?}", err);
                Err(StatusCode::NOT_FOUND)
            },
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_ledger(session: &Session, field: &str, value: &str) -> Result<LedgerScylla> {
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
    let ledger = query_result.single_row_typed::<LedgerScylla>()?;

    Ok(ledger)
}
