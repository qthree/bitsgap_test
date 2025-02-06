use anyhow::Context;
use bitsgap_shared::{
    interval::DatabaseIntervals,
    records::kline::{Kline, VBS},
    utils::Has,
};
use let_clone::let_clone;

use super::intervals::WsCandlesChannels;
use crate::units::{PxCount, PxPrice, PxSymbol, PxTimestamp, PxUnits};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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

impl CandlesMessage {
    // TODO: refactor into trait, something like `ToInternal`
    pub fn kline<C: Has<DatabaseIntervals> + Has<WsCandlesChannels>>(
        &self,
        channel: &str,
        context: &C,
    ) -> anyhow::Result<Kline> {
        let Self {
            symbol,
            amount,
            high,
            quantity,
            low,
            start_time,
            close,
            open,
            ..
        } = self;

        let interval = context
            .give(WsCandlesChannels)
            .to_interval(channel)
            .context("convert WS channel name to interval")?;

        let time_frame = context
            .give(DatabaseIntervals)
            .to_alias(interval)
            .context("convert interval to databse time frame format")?
            .into();

        let_clone!(symbol: pair);

        let amount: f64 = amount.parse().context("parse amount")?;
        let quantity: f64 = quantity.parse().context("parse quantity")?;

        // Can't figure out buy/sell from WS message, only total base/quote
        // "Достаточно заполнить buy_base: quantity"
        // let's fill data into "buy" for now.
        let volume_bs = VBS {
            buy_base: quantity,
            sell_base: 0.0,
            buy_quote: amount,
            sell_quote: 0.0,
        };

        Ok(Kline {
            pair,
            time_frame,
            o: open.parse().context("parse open price")?,
            h: high.parse().context("parse open price")?,
            l: low.parse().context("parse open price")?,
            c: close.parse().context("parse open price")?,
            utc_begin: (*start_time).try_into().context("convert start time")?,
            volume_bs,
        })
    }
}
