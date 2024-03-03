mod handlers;
mod models;
mod utils;

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
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("API listening on port 3000");
}
