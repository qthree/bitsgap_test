use anyhow::{Context, bail};
use bitsgap_shared::{
    Request,
    interval::{DatabaseIntervals, ExchangeIntervals, Interval},
    records::kline::{Kline, VBS},
    utils::{
        Has,
        url::{BuildUrl, UrlBuilder},
    },
};

use crate::units::{PxCount, PxInterval, PxPrice, PxTimestamp, PxUnits};

pub struct CandlesRequest<S> {
    /// symbol name
    pub symbol: S,
    /// the unit of time to aggregate data by
    pub interval: Interval,
    /// maximum number of records returned. The default value is 100 and the max value is 500
    pub limit: Option<u16>,
    /// filters by time
    /// the default value is 0
    pub start_time: Option<u64>,
    /// filters by time
    /// the default value is current time
    pub end_time: Option<u64>,
}

impl<S> Request for CandlesRequest<S> {
    type Response = Vec<CandlesResponse>;
}

impl<S: AsRef<str>, C: Has<ExchangeIntervals>> BuildUrl<C> for CandlesRequest<S> {
    fn build_url(&self, url_builder: &mut UrlBuilder, context: &C) -> anyhow::Result<()> {
        url_builder.add_path_segments(&["markets", self.symbol.as_ref(), "candles"])?;

        let supported_intervals = context.give(ExchangeIntervals);
        let Some(interval_alias) = supported_intervals.to_alias(self.interval) else {
            bail!("unsupported interval")
        };
        let mut query_builder = url_builder.query_builder()?;

        query_builder.add_pair("interval", interval_alias);
        if let Some(limit) = self.limit {
            query_builder.display_pair("limit", &limit)?;
        }
        if let Some(start_time) = self.start_time {
            query_builder.display_pair("startTime", &start_time)?;
        }
        if let Some(end_time) = self.end_time {
            query_builder.display_pair("endTime", &end_time)?;
        }
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CandlesResponse {
    /// lowest price over the interval
    pub low: PxPrice,
    /// highest price over the interval
    pub high: PxPrice,
    /// price at the start time
    pub open: PxPrice,
    /// price at the end time
    pub close: PxPrice,
    /// quote units traded over the interval
    pub amount: PxUnits,
    /// base units traded over the interval
    pub quantity: PxUnits,
    /// quote units traded over the interval filled by market buy orders
    pub buy_taker_amount: PxUnits,
    /// base units traded over the interval filled by market buy orders
    pub buy_taker_quantity: PxUnits,
    /// integer count of trades
    pub trade_count: PxCount,
    /// time the record was pushed
    /// it's usually later than `close_time`
    /// "ts" in official documentation
    pub record_time: PxTimestamp,
    /// weighted average over the interval
    pub weighted_average: PxPrice,
    /// the selected interval
    pub interval: PxInterval,
    /// start time of interval
    pub start_time: PxTimestamp,
    /// close time of interval
    pub close_time: PxTimestamp,
}

impl CandlesResponse {
    // TODO: refactor into trait, something like `ToInternal`
    pub fn kline<C: Has<DatabaseIntervals>, S: AsRef<str>>(
        &self,
        request: &CandlesRequest<S>,
        context: &C,
    ) -> anyhow::Result<Kline> {
        // TODO: check if intervals in request and response match?
        let CandlesRequest {
            ref symbol,
            interval,
            ..
        } = *request;
        let CandlesResponse {
            low,
            high,
            open,
            close,
            amount,
            quantity,
            buy_taker_amount,
            buy_taker_quantity,
            start_time,
            ..
        } = self;

        // Right now, it does match to database format from test task, but we need think about converting it in the futures
        let pair = symbol.as_ref().into();

        let time_frame = context
            .give(DatabaseIntervals)
            .to_alias(interval)
            .context("convert interval to databse time frame format")?
            .into();

        // TODO: make type which deserializes as String, but stores in-memory as number, use it in CandlesResponse
        // TODO: verify numbers to be positive and finite
        let amount: f64 = amount.parse().context("parse amount")?;
        let quantity: f64 = quantity.parse().context("parse quantity")?;
        let buy_taker_amount: f64 = buy_taker_amount.parse().context("parse buy taker amount")?;
        let buy_taker_quantity: f64 = buy_taker_quantity
            .parse()
            .context("parse buy taker quantity")?;

        let volume_bs = VBS {
            buy_base: buy_taker_quantity,
            sell_base: quantity - buy_taker_quantity,
            buy_quote: buy_taker_amount,
            sell_quote: amount - buy_taker_amount,
        };
        if volume_bs.sell_base < 0.0 {
            bail!("sell base volume is negative");
        }
        if volume_bs.sell_quote < 0.0 {
            bail!("sell quote volume is negative");
        }

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
