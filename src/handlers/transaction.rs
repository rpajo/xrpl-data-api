use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use scylla::Session;
use scylla::transport::session::TypedRowIter;
use crate::AppState;
use crate::consts::{KEYSPACE, TRANSACTIONS_TABLE};
use crate::models::transaction::Transaction;

pub async fn get_transaction_by_hash(
    State(state): State<Arc<AppState>>,
    Path(tx_hash): Path<String>
) -> anyhow::Result<Json<Transaction>, StatusCode> {
    let match_value = format!("'{}'", tx_hash);
    match get_transactions(&state.scylla_session, "hash", &match_value.to_string()).await {
        Ok(mut transactions) => Ok(Json(transactions.next().unwrap().unwrap())),
        Err(err) => {
            eprintln!("Error fetching transaction: {:?}", err);
            Err(StatusCode::NOT_FOUND)
        },
    }
}
async fn get_transactions(session: &Session, field: &str, value: &str) -> anyhow::Result<TypedRowIter<Transaction>> {
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
            from \"{}\".{} WHERE {}={};",
        KEYSPACE, TRANSACTIONS_TABLE, field, value
    );
    println!("Query: {}", query);
    let query_result = session.query(query, &[]).await?;
    let transactions_iter = query_result.rows_typed::<Transaction>()?;

    // let mut transactions = Vec::new();
    // for row in transactions_iter {
    //     match row {
    //         Ok(tx) => transactions.push(tx),
    //         Err(err) => eprintln!("Failed to get tx: {:?}", err)
    //     }
    // }

    Ok(transactions_iter)
}