use bitsgap_shared::interval::{IntervalKind, SupportedIntervals};

pub mod candles;

pub const TEST_SYMBOLS: &[&str] = &["BTC_USDT", "TRX_USDT", "ETH_USDT", "DOGE_USDT", "BCH_USDT"];

// TOOD: load from config
pub fn supported_intervals() -> anyhow::Result<SupportedIntervals> {
    SupportedIntervals::default()
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
    use core::fmt;

    use bitsgap_shared::{
        ApiConfig, ApiFactory, ApiRequester, AuthMethod, Request,
        interval::Interval,
        utils::{Has, url::BuildUrl},
    };

    use super::*;

    struct Context {
        supported_intervals: SupportedIntervals,
    }

    impl Has<SupportedIntervals> for Context {
        fn give(&self) -> &SupportedIntervals {
            &self.supported_intervals
        }
    }

    fn poloniex_requester() -> ApiRequester<Context> {
        let api_key = std::env::var("API_KEY").expect("API_KEY env var");
        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
        let context = Context {
            supported_intervals: super::supported_intervals().unwrap(),
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

    async fn poloniex_get_and_print_json(path: impl BuildUrl<Context>) {
        let value: serde_json::Value = poloniex_requester().get_json(path).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    }

    async fn poloniex_get_response_and_print_debug<
        R: Request<Response: serde::de::DeserializeOwned + fmt::Debug> + BuildUrl<Context>,
    >(
        path: R,
    ) {
        let response = poloniex_requester().get_response(path).await.unwrap();
        println!("{response:#?}");
    }

    #[tokio::test]
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
                limit: Some(10),
                start_time: Some(1738700743 * 1000),
                end_time: Some(1738770743 * 1000),
            };
            poloniex_get_response_and_print_debug(req).await;
        }
    }

    #[test]
    fn test_poliniex_candles_url() {
        let url = poloniex_requester()
            .build_url(candles::CandlesRequest {
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
