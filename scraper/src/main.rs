use std::collections::BTreeMap;

use anyhow::Context;
use bitsgap_poloniex::{
    context::PoloniexContext,
    ws::{channels::Channel, intervals::WsCandlesChannels},
};
use bitsgap_shared::{
    ApiConfig, ApiFactory, AuthMethod,
    utils::{Has, time::timestamp_parse},
};
use clap::Parser;
use storage::Storage;

mod download;
mod storage;
mod stream;

#[derive(Debug, Parser)]
struct Config {
    #[arg(env, long)]
    api_key: String,
    #[arg(env, long)]
    secret_key: String,
    /// MongoDB URI
    #[arg(env, long)]
    mongodb_uri: String,
    /// Download KL since timestamp
    #[arg(long)]
    since: String,
    /// Download KL limit per interval
    #[arg(long = "download-limit")]
    download_limit_per_interval: Option<u32>,
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
        since,
        download_limit_per_interval,
    } = Config::parse();
    let since = timestamp_parse(&since)?;

    let storage = Storage::init(&mongodb_uri).await.context("init storage")?;

    scrap_poloniex(
        api_key,
        secret_key,
        storage,
        since,
        download_limit_per_interval,
    )
    .await
}

async fn scrap_poloniex(
    api_key: String,
    secret_key: String,
    storage: Storage,
    since: u64,
    download_limit_per_interval: Option<u32>,
) -> anyhow::Result<()> {
    // TODO: move to config
    let base_url = "https://api.poloniex.com"
        .try_into()
        .context("parse exchange api url")?;

    let context = PoloniexContext::init(true).context("init poloniex context")?;
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

    let symbols = bitsgap_poloniex::TEST_TASK_SYMBOLS;

    download::poloniex_klines(
        &requester,
        &storage,
        symbols,
        since,
        500,
        download_limit_per_interval,
    )
    .await
    .context("download klines")?;

    let context = requester.context();

    // TODO: use SortedVec?
    let mut channels = BTreeMap::new();
    channels.extend(
        context
            .give(WsCandlesChannels)
            .iter()
            .map(|(interval, alias)| (alias.to_string(), Channel::Candles(interval))),
    );
    channels.insert("trades".into(), Channel::Trades);

    stream::dump_events(requester.context(), &storage, None, channels, symbols)
        .await
        .context("dump events")?;
    Ok(())
}
