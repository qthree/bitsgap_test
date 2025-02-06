use anyhow::Context;
use bitsgap_poloniex::{context::PoloniexContext, rest::candles::CandlesRequest};
use bitsgap_shared::{
    ApiConfig, ApiFactory, ApiRequester, AuthMethod,
    interval::DatabaseIntervals,
    records::{kline::Kline, recent_trade::RecentTrade},
    utils::{
        Has,
        time::{timestamp_ceil, timestamp_display, timestamp_parse},
    },
};
use clap::Parser;
use mongodb::{Collection, IndexModel, bson::doc, options::IndexOptions};

#[derive(Debug, Parser)]
struct Config {
    #[arg(env)]
    api_key: String,
    #[arg(env)]
    secret_key: String,
    #[arg(env, long)]
    mongodb_uri: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();
    let Config {
        api_key,
        secret_key,
        mongodb_uri,
    } = Config::parse();

    // Mongo is temporary solution, no need for abstractions and refactoring into shared crate
    let mongo_options = mongodb::options::ClientOptions::parse(&mongodb_uri)
        .await
        .context("parse and resolve mongodb uri")?;
    let mongo_client =
        mongodb::Client::with_options(mongo_options).context("create mongodb client")?;
    let database = mongo_client
        .default_database()
        .context("default database is not set")?;
    //let database = mongo_client.database("bitsgap_qthree_test");

    // Clean up data from last run
    database.drop().await.context("drop database")?;

    // Index options
    let options = Some(IndexOptions::builder().unique(true).build());

    // Create indices
    let klines = database.collection("klines");
    klines
        .create_index(
            IndexModel::builder()
                .keys(doc! { "pair": 1, "time_frame": 1, "utc_begin": 1})
                .options(options.clone())
                .build(),
        )
        .await
        .context("create klines index")?;
    let recent_trades = database.collection("recent_trades");
    recent_trades
        .create_index(
            IndexModel::builder()
                .keys(doc! { "pair": 1, "tid": 1})
                .options(options.clone())
                .build(),
        )
        .await
        .context("create recent_trades index")?;

    scrap_poloniex(
        api_key,
        secret_key,
        Storage {
            klines,
            recent_trades,
        },
    )
    .await
}

struct Storage {
    klines: Collection<Kline>,
    recent_trades: Collection<RecentTrade>,
}
type OneOrMany<T> = smallvec::SmallVec<[T; 1]>;

impl Storage {
    async fn insert_klines(&self, klines: OneOrMany<Kline>) -> anyhow::Result<()> {
        self.klines
            .insert_many(&klines)
            .await
            .context("insert klines into storage")?;
        Ok(())
    }

    async fn insert_recent_trades(
        &self,
        recent_trades: OneOrMany<RecentTrade>,
    ) -> anyhow::Result<()> {
        self.recent_trades
            .insert_many(&recent_trades)
            .await
            .context("insert recent trades into storage")?;
        Ok(())
    }

    async fn count_klines(&self) -> anyhow::Result<u64> {
        self.klines
            .count_documents(Default::default())
            .await
            .context("count klines in storage")
    }
}

async fn scrap_poloniex(
    api_key: String,
    secret_key: String,
    storage: Storage,
) -> anyhow::Result<()> {
    // TODO: move to config
    //let since = timestamp_parse("2024-12-01T00:00:00Z")?;
    let since = timestamp_parse("2025-02-01T00:00:00Z")?;
    let base_url = "https://api.poloniex.com"
        .try_into()
        .context("parse exchange api url")?;

    let context = PoloniexContext::init();
    let requester = ApiFactory::new().make_requester(
        ApiConfig {
            base_url,
            auth: AuthMethod::HmacSha256 {
                api_key,
                secret_key,
            },
        },
        context,
    );
    download_poloniex_klines(&requester, &storage, since, 500)
        .await
        .context("download klines")?;
    Ok(())
}

async fn download_poloniex_klines(
    requester: &ApiRequester<PoloniexContext>,
    storage: &Storage,
    since: u64,
    limit: u16,
) -> anyhow::Result<()> {
    // Download historic klines
    log::info!("Downloading historic klines...");
    let mut total_klines_downloaded = 0;
    for symbol in bitsgap_poloniex::TEST_TASK_SYMBOLS {
        let intervals = requester.context().give(DatabaseIntervals).iter_intervals();
        for interval in intervals {
            let mut start_time = since;
            // TODO: should use new "shared" request type, once I'll figure out proper abstraction between different exchanges
            // TODO: should automatically convert to klines
            loop {
                println!(
                    "interval: {interval:?}, start_time: {}",
                    timestamp_display(start_time)
                );
                let req = CandlesRequest {
                    symbol,
                    interval,
                    // max value is 500
                    limit: Some(limit),
                    start_time: Some(start_time),
                    end_time: None,
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
                        "Downloaded {count} klines, start: {}, end: {}",
                        timestamp_display(first.start_time),
                        timestamp_display(last.close_time)
                    );
                    start_time = timestamp_ceil(last.close_time);
                }

                total_klines_downloaded += count;
                if count < limit as _ {
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
