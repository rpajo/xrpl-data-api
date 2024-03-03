use chrono::Utc;
use num_bigint::BigInt;
use scylla::{
    frame::value::{CqlDate, CqlTimestamp},
    FromRow,
};
use serde::{ser::SerializeStruct, Serialize, Serializer};

#[derive(Debug, FromRow)]
pub struct DailyLedgerScylla {
    pub ledger_close_day: CqlDate,
    pub ledger_index: i64,
    pub close_time: CqlTimestamp,
}

#[derive(Debug, FromRow)]
pub struct LedgerScylla {
    pub ledger_index: i64,
    pub ledger_hash: String,
    pub account_hash: String,
    pub parent_hash: String,
    pub transaction_hash: String,
    pub close_flags: i32,
    pub close_time: chrono::DateTime<Utc>,
    pub parent_close_time: chrono::DateTime<Utc>,
    pub total_coins: i64,
    pub tx_count: BigInt,
    pub ledger_processed: bool,
}

impl Serialize for LedgerScylla {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("LedgerScylla", 11)?;
        state.serialize_field("ledger_index", &self.ledger_index)?;
        state.serialize_field("ledger_hash", &self.ledger_hash)?;
        state.serialize_field("parent_hash", &self.parent_hash)?;
        state.serialize_field("account_hash", &self.account_hash)?;
        state.serialize_field("transaction_hash", &self.transaction_hash)?;
        state.serialize_field("close_flags", &self.close_flags)?;
        state.serialize_field("close_time", &self.close_time.format("%+").to_string())?;
        state.serialize_field(
            "parent_close_time",
            &self.parent_close_time.format("%+").to_string(),
        )?;
        state.serialize_field("total_coins", &self.total_coins)?;
        state.serialize_field("tx_count", &self.tx_count.to_string())?;
        // state.serialize_field("ledger_processed", &self.ledger_processed)?;
        state.end()
    }
}
