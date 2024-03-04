use crate::models::account::Account;
use crate::utils::consts::ACCOUNTS_TABLE;
use crate::utils::errors::{map_error_to_status_code, DataApiError};
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use scylla::Session;
use std::sync::Arc;

pub async fn get_account_handler(
    State(state): State<Arc<AppState>>,
    Path(account): Path<String>,
) -> anyhow::Result<Json<Account>, StatusCode> {
    match get_account(&state.scylla_session, &account).await {
        Ok(account) => Ok(Json(account)),
        Err(err) => {
            eprintln!("{}", err);
            Err(map_error_to_status_code(&err))
        }
    }
}

async fn get_account(session: &Session, account: &str) -> Result<Account, DataApiError> {
    let query = format!(
        "SELECT ledger_index, \
            tx_index, \
            account, \
            client, \
            initial_balance, \
            parent, \
            timestamp, \
            tx_hash \
        from {} WHERE account=?;",
        ACCOUNTS_TABLE
    );
    println!("Query: {}", query);
    let query_result = session.query(query, (account,)).await?;
    if let Ok(num_of_rows) = query_result.rows_num() {
        if num_of_rows == 0 {
            return Err(DataApiError::NoDataReturned);
        }
    }
    let account = query_result.single_row_typed::<Account>()?;
    Ok(account)
}
