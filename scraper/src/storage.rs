use anyhow::Context as _;
use bitsgap_shared::records::{kline::Kline, recent_trade::RecentTrade};
use mongodb::{
    Collection, IndexModel,
    bson::{self, doc},
    options::IndexOptions,
};

pub(crate) struct Storage {
    klines: Collection<Kline>,
    recent_trades: Collection<RecentTrade>,
}
pub(crate) type OneOrMany<T> = smallvec::SmallVec<[T; 1]>;

// Mongo is temporary solution, no need for excessive abstractions and refactoring into shared crate
impl Storage {
    pub(crate) async fn init(mongodb_uri: &str) -> anyhow::Result<Self> {
        let mongo_options = mongodb::options::ClientOptions::parse(mongodb_uri)
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

        Ok(Storage {
            klines,
            recent_trades,
        })
    }

    pub(crate) async fn insert_klines(&self, klines: OneOrMany<Kline>) -> anyhow::Result<usize> {
        let len = klines.len();
        self.klines
            .insert_many(&klines)
            .await
            .context("insert klines into storage")?;
        Ok(len)
    }

    pub(crate) async fn upsert_kline(&self, kline: Kline) -> anyhow::Result<usize> {
        let Kline {
            pair,
            time_frame,
            o,
            h,
            l,
            c,
            utc_begin,
            volume_bs,
        } = kline;
        let volume_bs = bson::to_bson(&volume_bs).context("volume_bs to bson")?;
        self.klines
            .update_one(
                doc! {"pair": pair, "time_frame": time_frame, "utc_begin": utc_begin},
                doc! {"$set": {"o": o, "h": h, "l": l, "c": c, "volume_bs": volume_bs}},
            )
            .upsert(true)
            .await
            .context("insert klines into storage")?;
        Ok(1)
    }

    pub(crate) async fn insert_recent_trades(
        &self,
        recent_trades: OneOrMany<RecentTrade>,
    ) -> anyhow::Result<usize> {
        let len = recent_trades.len();
        self.recent_trades
            .insert_many(&recent_trades)
            .await
            .context("insert recent trades into storage")?;
        Ok(len)
    }

    pub(crate) async fn count_klines(&self) -> anyhow::Result<u64> {
        self.klines
            .count_documents(Default::default())
            .await
            .context("count klines in storage")
    }
}
