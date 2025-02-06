pub mod rest;
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
    use ws::intervals::ws_candles_channels;

    use super::*;

    #[allow(dead_code)]
    pub(crate) struct Context {
        exchange_intervals: IntervalsDict,
        ws_candles_channels: IntervalsDict,
        database_intervals: IntervalsDict,
    }

    impl Has<ExchangeIntervals> for Context {
        fn give(&self) -> &IntervalsDict {
            &self.exchange_intervals
        }
    }

    impl Has<DatabaseIntervals> for Context {
        fn give(&self) -> &IntervalsDict {
            &self.database_intervals
        }
    }

    pub(crate) fn poloniex_requester() -> ApiRequester<Context> {
        let api_key = std::env::var("API_KEY").expect("API_KEY env var");
        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
        let context = Context {
            exchange_intervals: exchange_intervals().unwrap(),
            ws_candles_channels: ws_candles_channels().unwrap(),
            database_intervals: database_intervals().unwrap(),
        };
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
