use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use scylla::FromRow;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;// Assuming you're using the chrono crate for handling timestamps

#[derive(Debug, FromRow)]
pub struct Transaction {
    pub account: String,
    pub hash: String,
    pub ctid: String,
    pub ledger_index: i64,
    pub tx_index: BigInt,
    pub tx_type: String,
    pub timestamp: DateTime<Utc>,
    pub flags: i64,
    pub fee: BigInt,
    pub sequence: i64,
    pub result: i16,
    pub meta: Vec<u8>,
    pub tx: Vec<u8>,
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut s = serializer.serialize_struct("Transaction", 13)?;
        s.serialize_field("account", &self.account)?;
        s.serialize_field("hash", &self.hash)?;
        s.serialize_field("ctid", &self.ctid)?;
        s.serialize_field("ledger_index", &self.ledger_index)?;
        s.serialize_field("tx_index", &self.tx_index.to_string())?;
        s.serialize_field("tx_type", &self.tx_type)?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.serialize_field("flags", &self.flags)?;
        s.serialize_field("fee", &self.fee.to_string())?;
        s.serialize_field("sequence", &self.sequence)?;
        s.serialize_field("result", &self.result)?;
        s.serialize_field("meta", &String::from_utf8_lossy(&self.meta).to_string())?;
        s.serialize_field("tx", &String::from_utf8_lossy(&self.tx).to_string())?;
        s.end()
    }
}