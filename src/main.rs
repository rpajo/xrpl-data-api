mod handlers;
mod models;
mod utils;

use crate::utils::consts::KEYSPACE;
use axum::{routing::get, Router};
use scylla::{Session, SessionBuilder};
use std::sync::Arc;

struct AppState {
    scylla_session: Arc<Session>,
}

#[tokio::main]
async fn main() {
    println!("Connecting to scylla.");

    let session: Session = SessionBuilder::new()
        .known_node("172.27.0.2")
        .known_node("172.27.0.3")
        .known_node("172.27.0.4")
        .build()
        .await
        .expect("Failed to connect to scylla nodes");

    session
        .use_keyspace(KEYSPACE, true)
        .await
        .expect("Unable to use keyspace");

    /* TODO: Prepare query statements beforehand,
    also string into queries can cause sql injections */
    println!("Connected.");

    let arc_session = Arc::new(session);

    let shared_state = Arc::new(AppState {
        scylla_session: arc_session,
    });

    // todo: sql injections for strings
    let app = Router::new()
        // Ledger handlers
        .route(
            "/ledger/:ledger_identifier",
            get(handlers::ledger::get_ledger_handler),
        )
        .route(
            "/daily_ledgers/:close_day",
            get(handlers::daily_ledger::get_daily_ledgers_handler),
        )
        .route(
            "/closed_ledger/:close_time",
            get(handlers::daily_ledger::get_latest_closed_ledger_handler),
        )
        // Transaction handlers
        .route(
            "/transaction/hash/:tx_hash",
            get(handlers::transaction::get_transaction_by_hash),
        )
        .route(
            "/transaction/ledger/:ledger_index",
            get(handlers::transaction::get_transaction_by_ledger_index),
        )
        .route(
            "/transaction/account/:account",
            get(handlers::transaction::get_transaction_by_account),
        )
        // Account handlers
        .route(
            "/account/:account",
            get(handlers::account::get_account_handler),
        )
        // Balance changes handlers
        .route(
            "/account/:account/balance_changes",
            get(handlers::balance_change::get_account_balance_changes_handler),
        )
        // Payments handlers
        .route(
            "/account/:account/payments",
            get(handlers::payment::get_account_payments_handler),
        )
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("API listening on port 3000");
}
