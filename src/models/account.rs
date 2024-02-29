use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use scylla::FromRow;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

#[derive(Debug, FromRow)]
pub struct Account {
    pub ledger_index: i64,
    pub tx_index: BigInt,
    pub account: String,
    pub client: Option<String>,
    pub initial_balance: String,
    pub parent: String,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: String,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut s = serializer.serialize_struct("Transaction", 8)?;
        s.serialize_field("ledger_index", &self.ledger_index)?;
        s.serialize_field("tx_index", &self.tx_index.to_string())?;
        s.serialize_field("account", &self.account)?;
        s.serialize_field("client", &self.client)?;
        s.serialize_field("initial_balance", &self.initial_balance)?;
        s.serialize_field("parent", &self.parent)?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.serialize_field("tx_hash", &self.tx_hash)?;
        s.end()
    }
}