use crate::units::{PxCount, PxPrice, PxSymbol, PxTimestamp, PxUnits};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlesMessage {
    /// symbol name
    pub symbol: PxSymbol,
    /// quote units traded over the interval
    pub amount: PxPrice,
    /// highest price over the interval
    pub high: PxPrice,
    /// base units traded over the interval
    pub quantity: PxUnits,
    /// count of trades
    pub trade_count: PxCount,
    /// lowest price over the interval
    pub low: PxPrice,
    /// close time of interval
    pub close_time: PxTimestamp,
    /// start time of interval
    pub start_time: PxTimestamp,
    /// price at the end time
    pub close: PxPrice,
    /// price at the start time
    pub open: PxPrice,
    /// time the record was pushed
    /// it's usually later than `close_time`
    #[serde(rename = "ts")]
    pub record_time: PxTimestamp,
}
