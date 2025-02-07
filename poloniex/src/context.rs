use anyhow::Context;
use bitsgap_shared::{
    interval::{DatabaseIntervals, ExchangeIntervals, IntervalsDict, database_intervals},
    utils::Has,
};

use crate::{
    rest::intervals::exchange_intervals,
    ws::intervals::{WsCandlesChannels, all_ws_candles_channels, supported_ws_candles_channels},
};

pub struct PoloniexContext {
    exchange_intervals: IntervalsDict,
    ws_candles_channels: IntervalsDict,
    database_intervals: IntervalsDict,
}
impl PoloniexContext {
    pub fn init(only_supported_candles: bool) -> anyhow::Result<Self> {
        let database_intervals = database_intervals().context("database intervals")?;
        Ok(Self {
            exchange_intervals: exchange_intervals().context("exchange intervals")?,
            ws_candles_channels: if only_supported_candles {
                supported_ws_candles_channels(&database_intervals)
            } else {
                all_ws_candles_channels()
            }
            .context("candles channels intervals")?,
            database_intervals,
        })
    }
}

impl Has<ExchangeIntervals> for PoloniexContext {
    fn give(&self, _label: ExchangeIntervals) -> &IntervalsDict {
        &self.exchange_intervals
    }
}

impl Has<DatabaseIntervals> for PoloniexContext {
    fn give(&self, _label: DatabaseIntervals) -> &IntervalsDict {
        &self.database_intervals
    }
}

impl Has<WsCandlesChannels> for PoloniexContext {
    fn give(&self, _label: WsCandlesChannels) -> &IntervalsDict {
        &self.ws_candles_channels
    }
}
