use bitsgap_shared::{
    interval::{DatabaseIntervals, ExchangeIntervals, IntervalsDict, database_intervals},
    utils::Has,
};

use crate::{
    rest::intervals::exchange_intervals,
    ws::intervals::{WsCandlesChannels, ws_candles_channels},
};

pub struct PoloniexContext {
    exchange_intervals: IntervalsDict,
    ws_candles_channels: IntervalsDict,
    database_intervals: IntervalsDict,
}
impl PoloniexContext {
    pub fn init() -> Self {
        Self {
            exchange_intervals: exchange_intervals().unwrap(),
            ws_candles_channels: ws_candles_channels().unwrap(),
            database_intervals: database_intervals().unwrap(),
        }
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
