pub mod rest;
pub mod units;
pub mod ws;

pub const TEST_TASK_SYMBOLS: &[&str] =
    &["BTC_USDT", "TRX_USDT", "ETH_USDT", "DOGE_USDT", "BCH_USDT"];

#[cfg(test)]
mod tests {
    use bitsgap_shared::{
        ApiConfig, ApiFactory, ApiRequester, AuthMethod,
        interval::{DatabaseIntervals, ExchangeIntervals, IntervalsDict, database_intervals},
        utils::Has,
    };
    use rest::intervals::exchange_intervals;
    use ws::intervals::{WsCandlesChannels, ws_candles_channels};

    use super::*;

    #[allow(dead_code)]
    pub(crate) struct PoloniexContext {
        exchange_intervals: IntervalsDict,
        ws_candles_channels: IntervalsDict,
        database_intervals: IntervalsDict,
    }
    impl PoloniexContext {
        pub(crate) fn init() -> Self {
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

    pub(crate) fn poloniex_requester() -> ApiRequester<PoloniexContext> {
        let api_key = std::env::var("API_KEY").expect("API_KEY env var");
        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
        let context = PoloniexContext::init();
        ApiFactory::new().make_requester(
            ApiConfig {
                base_url: "https://api.poloniex.com".try_into().unwrap(),
                auth: AuthMethod::HmacSha256 {
                    api_key,
                    secret_key,
                },
            },
            context,
        )
    }
}
