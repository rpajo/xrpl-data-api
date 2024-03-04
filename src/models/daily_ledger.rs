use chrono::{DateTime, NaiveDate, Utc};
use scylla::FromRow;
use serde::Serialize;

#[derive(Debug, Serialize, FromRow)]
pub struct DailyLedger {
    #[serde(skip_serializing)]
    pub ledger_close_day: NaiveDate,
    pub ledger_index: i64,
    pub close_time: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub close_time_unix: i64,
}
