use bitsgap_shared::interval::{IntervalKind, IntervalsDict};

pub mod candles;

pub const TEST_SYMBOLS: &[&str] = &["BTC_USDT", "TRX_USDT", "ETH_USDT", "DOGE_USDT", "BCH_USDT"];

// TOOD: load from config
pub fn exchange_intervals() -> anyhow::Result<IntervalsDict> {
    IntervalsDict::default()
        .with(
            IntervalKind::Minute,
            [
                (1, "MINUTE_1"),
                (5, "MINUTE_5"),
                (10, "MINUTE_10"),
                (15, "MINUTE_15"),
                (30, "MINUTE_30"),
            ],
        )?
        .with(
            IntervalKind::Hour,
            [
                (1, "HOUR_1"),
                (2, "HOUR_2"),
                (4, "HOUR_4"),
                (6, "HOUR_6"),
                (12, "HOUR_12"),
            ],
        )?
        .with(IntervalKind::Day, [(1, "DAY_1"), (3, "DAY_3")])?
        .with(IntervalKind::Week, [(1, "WEEK_1")])?
        .with(IntervalKind::Month, [(1, "MONTH_1")])
}

#[cfg(test)]
mod tests {
    use bitsgap_shared::{
        ApiConfig, ApiFactory, ApiRequester, AuthMethod,
        interval::{DatabaseIntervals, ExchangeIntervals, Interval, database_intervals},
        utils::{Has, url::BuildUrl},
    };

    use super::*;

    struct Context {
        exchange_intervals: IntervalsDict,
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

    fn poloniex_requester() -> ApiRequester<Context> {
        let api_key = std::env::var("API_KEY").expect("API_KEY env var");
        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
        let context = Context {
            exchange_intervals: super::exchange_intervals().unwrap(),
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

    async fn poloniex_get_and_print_json<B: ?Sized + BuildUrl<Context>>(path: &B) {
        let value: serde_json::Value = poloniex_requester().get_json(path).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn get_poloniex_markets() {
        poloniex_get_and_print_json("markets").await;
    }

    #[tokio::test]
    async fn get_poloniex_candles() {
        for symbol in super::TEST_SYMBOLS {
            let req = candles::CandlesRequest {
                symbol,
                interval: Interval {
                    kind: IntervalKind::Minute,
                    value: 1,
                },
                limit: Some(1),
                start_time: Some(1738700743 * 1000),
                end_time: Some(1738770743 * 1000),
            };
            let requester = poloniex_requester();
            let responses = requester.get_response(&req).await.unwrap();
            for response in responses {
                let kline = response.kline(&req, requester.context()).unwrap();
                println!("{}", serde_json::to_string(&kline).unwrap());
            }
        }
    }

    #[test]
    fn test_poliniex_candles_url() {
        let url = poloniex_requester()
            .build_url(&candles::CandlesRequest {
                symbol: "BTC_USDT",
                interval: Interval {
                    kind: IntervalKind::Minute,
                    value: 1,
                },
                limit: Some(10),
                start_time: Some(1738700743 * 1000),
                end_time: Some(1738770743 * 1000),
            })
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://api.poloniex.com/markets/BTC_USDT/candles?interval=MINUTE_1&limit=10&startTime=1738700743000&endTime=1738770743000"
        );
    }
}
