use anyhow::bail;
use bitsgap_shared::{
    Request,
    interval::{Interval, SupportedIntervals},
    utils::{
        Has,
        url::{BuildUrl, UrlBuilder},
    },
};

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

impl<S: AsRef<str>, C: Has<SupportedIntervals>> BuildUrl<C> for CandlesRequest<S> {
    fn build_url(&self, url_builder: &mut UrlBuilder, context: &C) -> anyhow::Result<()> {
        url_builder.add_path_segments(&["markets", self.symbol.as_ref(), "candles"])?;

        let supported_intervals = context.give();
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
    pub low: String,
    /// highest price over the interval
    pub high: String,
    /// price at the start time
    pub open: String,
    /// price at the end time
    pub close: String,
    /// quote units traded over the interval
    pub amount: String,
    /// base units traded over the interval
    pub quantity: String,
    /// quote units traded over the interval filled by market buy orders
    pub buy_taker_amount: String,
    /// base units traded over the interval filled by market buy orders
    pub buy_taker_quantity: String,
    /// integer count of trades
    pub trade_count: u32,
    /// time the record was pushed
    pub timestamp: u64,
    /// weighted average over the interval
    pub weighted_average: String,
    /// the selected interval
    pub interval: String,
    /// start time of interval
    pub start_time: u64,
    /// close time of interval
    pub close_time: u64,
}
