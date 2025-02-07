use anyhow::Context as _;
use bitsgap_poloniex::{context::PoloniexContext, rest::candles::CandlesRequest};
use bitsgap_shared::{
    ApiRequester,
    interval::DatabaseIntervals,
    utils::{Has, time::timestamp_display},
};

use crate::storage::Storage;

pub(crate) async fn poloniex_klines(
    requester: &ApiRequester<PoloniexContext>,
    storage: &Storage,
    symbols: &[&str],
    since: u64,
    limit_per_request: u16,
    total_limit: Option<usize>,
) -> anyhow::Result<()> {
    // Download historic klines
    log::info!("Downloading historic klines...");
    let mut total_klines_downloaded = 0;
    for symbol in symbols {
        let intervals = requester.context().give(DatabaseIntervals).iter();
        for (interval, interval_name) in intervals {
            let mut end_time = None;
            // TODO: should use new "shared" request type, once I'll figure out proper abstraction between different exchanges
            // TODO: should automatically convert to klines
            loop {
                let req = CandlesRequest {
                    symbol,
                    interval,
                    // max value is 500
                    limit: Some(limit_per_request),
                    start_time: Some(since),
                    end_time,
                };
                let responses = requester
                    .get_response(&req)
                    .await
                    .context("get candles response from rest api")?;
                let klines = responses
                    .iter()
                    .map(|response| response.kline(&req, requester.context()))
                    .collect::<anyhow::Result<_>>()
                    .context("convert candles to klines")?;
                storage.insert_klines(klines).await?;

                let count = responses.len();
                if let Some((first, last)) = responses.first().zip(responses.last()) {
                    log::info!(
                        "Downloaded {count} klines, symbol: {symbol}, interval: {interval_name:?}, start: {}, end: {}",
                        timestamp_display(first.start_time),
                        timestamp_display(last.close_time)
                    );
                    end_time = Some(first.start_time - 1);
                }

                total_klines_downloaded += count;
                if count < limit_per_request as _ {
                    break;
                }
                if matches!(total_limit, Some(total_limit) if total_klines_downloaded >= total_limit)
                {
                    break;
                }
            }
        }
    }

    let klines_in_storage = storage.count_klines().await?;
    log::info!(
        "Downloaded {total_klines_downloaded} klines. Storage has {klines_in_storage} klines."
    );

    Ok(())
}
