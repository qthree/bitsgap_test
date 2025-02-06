pub mod candles;
pub mod intervals;

#[cfg(test)]
mod tests {
    use bitsgap_shared::{
        interval::{Interval, IntervalKind},
        utils::url::BuildUrl,
    };

    use super::candles::CandlesRequest;
    use crate::{context::PoloniexContext, tests::poloniex_requester};

    async fn poloniex_get_and_print_json<B: BuildUrl<PoloniexContext>>(path: &B) {
        let value: serde_json::Value = poloniex_requester().get_json(path).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn get_poloniex_markets() {
        poloniex_get_and_print_json(&"markets").await;
    }

    #[tokio::test]
    async fn get_poloniex_trades() {
        poloniex_get_and_print_json(&&["markets", "BTC_USDT", "trades"]).await;
    }

    #[tokio::test]
    #[ignore]
    async fn get_poloniex_candles() {
        let requester = poloniex_requester();
        for symbol in crate::TEST_TASK_SYMBOLS {
            let req = CandlesRequest {
                symbol,
                interval: Interval {
                    kind: IntervalKind::Minute,
                    value: 1,
                },
                limit: Some(1),
                start_time: Some(1738700743 * 1000),
                end_time: Some(1738770743 * 1000),
            };
            let responses = requester.get_response(&req).await.unwrap();
            for response in responses {
                let kline = response.kline(&req, requester.context()).unwrap();
                println!("{}", serde_json::to_string(&kline).unwrap());
            }
        }
    }

    #[test]
    #[ignore]
    fn test_poliniex_candles_url() {
        let url = poloniex_requester()
            .build_url(&CandlesRequest {
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
