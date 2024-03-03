use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use scylla::FromRow;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, FromRow)]
pub struct BalanceChange {
    pub ledger_index: i64,
    pub tx_index: BigInt,
    pub node_index: BigInt,
    pub account: String,
    pub change: String,
    pub change_type: String,
    pub counterparty: Option<String>,
    pub currency: String,
    pub final_balance: String,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: String,
}

impl Serialize for BalanceChange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("BalanceChange", 11)?;
        s.serialize_field("ledger_index", &self.ledger_index)?;
        s.serialize_field("tx_index", &self.tx_index.to_string())?;
        s.serialize_field("node_index", &self.node_index.to_string())?;
        s.serialize_field("account", &self.account)?;
        s.serialize_field("change", &self.change)?;
        s.serialize_field("change_type", &self.change_type)?;
        s.serialize_field("counterparty", &self.counterparty)?;
        s.serialize_field("currency", &self.currency)?;
        s.serialize_field("final_balance", &self.final_balance)?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.serialize_field("tx_hash", &self.tx_hash)?;
        s.end()
    }
}
