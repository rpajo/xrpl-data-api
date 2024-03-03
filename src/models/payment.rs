use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use scylla::FromRow;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

#[derive(Debug, FromRow)]
pub struct Payment {
    pub tx_hash: String,
    pub ledger_index: i64,
    pub tx_index: BigInt,
    pub source: String,
    pub source_currency: String,
    pub source_currency_issuer: String,
    pub destination: String,
    pub destination_currency: String,
    pub destination_currency_issuer: String,
    pub amount: String,
    pub delivered_amount: String,
    pub transaction_cost: BigInt,
    pub destination_tag: Option<i64>,
    pub source_tag: Option<i64>,
    pub timestamp: DateTime<Utc>,
}

impl Serialize for Payment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("Payment", 15)?;

        state.serialize_field("tx_hash", &self.tx_hash)?;
        state.serialize_field("ledger_index", &self.ledger_index)?;
        state.serialize_field("tx_index", &self.tx_index.to_string())?;
        state.serialize_field("source", &self.source)?;
        state.serialize_field("source_currency", &self.source_currency)?;
        state.serialize_field("source_currency_issuer", &self.source_currency_issuer)?;
        state.serialize_field("destination", &self.destination)?;
        state.serialize_field("destination_currency", &self.destination_currency)?;
        state.serialize_field("destination_currency_issuer", &self.destination_currency_issuer)?;
        state.serialize_field("amount", &self.amount)?;
        state.serialize_field("delivered_amount", &self.delivered_amount)?;
        state.serialize_field("transaction_cost", &self.transaction_cost.to_string())?;
        state.serialize_field("destination_tag", &self.destination_tag)?;
        state.serialize_field("source_tag", &self.source_tag)?;
        state.serialize_field("timestamp", &self.timestamp)?;

        state.end()
    }
}