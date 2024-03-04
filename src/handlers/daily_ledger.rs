use crate::models::daily_ledger::DailyLedger;
use crate::utils::consts::DAILY_LEDGERS_TABLE;
use crate::utils::errors::{map_error_to_status_code, DataApiError};
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use scylla::Session;
use std::sync::Arc;

pub async fn get_daily_ledgers_handler(
    State(state): State<Arc<AppState>>,
    Path(close_day): Path<String>,
) -> Result<Json<Vec<DailyLedger>>, StatusCode> {
    let parsed_day = close_day.parse::<NaiveDate>();
    match parsed_day {
        Ok(day) => {
            println!("Finding ledgers closed in day: {}", day.format("%Y-%m-%d"));
            match get_ledgers_on_day(&state.scylla_session, day).await {
                Ok(ledgers) => Ok(Json(ledgers)),
                Err(err) => {
                    eprintln!("{}", err);
                    Err(map_error_to_status_code(&err))
                }
            }
        }
        Err(err) => {
            println!("Failed to parse day: {}", err);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn get_latest_closed_ledger_handler(
    State(state): State<Arc<AppState>>,
    Path(close_time): Path<String>,
) -> Result<Json<DailyLedger>, StatusCode> {
    let parsed_day = close_time.parse::<DateTime<Utc>>();
    match parsed_day {
        Ok(datetime) => {
            println!("Finding ledgers closed at: {}", datetime.format("%+"));
            match get_last_closed_ledger(&state.scylla_session, datetime).await {
                Ok(ledger) => Ok(Json(ledger)),
                Err(err) => {
                    eprintln!("{}", err);
                    Err(map_error_to_status_code(&err))
                }
            }
        }
        Err(err) => {
            println!("Failed to parse datetime: {}", err);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn get_ledgers_on_day(
    session: &Session,
    day: NaiveDate,
) -> Result<Vec<DailyLedger>, DataApiError> {
    let query = format!(
        "SELECT ledger_close_day,\
            ledger_index,\
            close_time, \
            toUnixTimestamp(close_time) \
        from {} WHERE ledger_close_day=?",
        DAILY_LEDGERS_TABLE,
    );
    println!("Query: {}", query);
    let query_result = session.query(query, (day,)).await?;
    let ledgers_iter = query_result.rows_typed_or_empty();

    // todo: better row error handling
    let ledgers = ledgers_iter
        .filter_map(|row| row.ok())
        .collect::<Vec<DailyLedger>>();

    if ledgers.is_empty() {
        return Err(DataApiError::NoDataReturned);
    }
    println!("Found {} closed ledgers", ledgers.len());

    Ok(ledgers)
}

pub async fn get_last_closed_ledger(
    session: &Session,
    close_time: DateTime<Utc>,
) -> Result<DailyLedger, DataApiError> {
    let query = format!(
        "SELECT ledger_close_day, \
            ledger_index, \
            close_time, \
            toUnixTimestamp(close_time) \
        from {} WHERE ledger_close_day = ?",
        DAILY_LEDGERS_TABLE,
        // close_time.date_naive()
    );
    let day_string = close_time.date_naive();
    println!("Query: {}", query);
    let query_result = session.query(query, (day_string,)).await?;
    let ledgers_iter = query_result.rows_typed_or_empty();

    // todo: better row error handling
    let ledger = ledgers_iter
        .filter_map(|row| row.ok())
        .max_by(|a: &DailyLedger, b: &DailyLedger| a.ledger_index.cmp(&b.ledger_index));

    match ledger {
        Some(latest) => Ok(latest),
        None => Err(DataApiError::NoDataReturned),
    }
}
