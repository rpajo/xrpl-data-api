mod models;
mod handlers;
mod consts;

use axum::{extract::State, routing::get, Router, Json};
use std::sync::Arc;
use axum::extract::Path;
use axum::http::StatusCode;
use scylla::{Session, SessionBuilder};
use serde_json::{json, Value};
use crate::models::ledger::LedgerScylla;


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
        scylla_session: arc_session
    });

    let app = Router::new()
        .route("/ledger/index/:ledger_index", get(handlers::ledger::ledger_by_index_handler))
        .route("/ledger/hash/:ledger_hash", get(handlers::ledger::ledger_by_hash_handler))
        .route("/ledger/time/:unix_time", get(handlers::ledger::ledger_at_time_handler))
        .route("/transaction/hash/:tx_hash", get(handlers::transaction::get_transaction_by_hash))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("API listening on port 3000");
}
