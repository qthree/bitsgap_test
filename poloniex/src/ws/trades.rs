use bitsgap_shared::records::recent_trade::RecentTrade;
use let_clone::let_clone;

use crate::units::{PxPrice, PxSymbol, PxTimestamp, PxUnits};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradesMessage {
    /// symbol name
    pub symbol: PxSymbol,
    /// quote units traded
    pub amount: PxPrice,
    /// trade side (buy, sell)
    pub taker_side: TakerSide,
    // base units traded
    pub quantity: PxUnits,
    /// time the trade was created
    pub create_time: PxTimestamp,
    /// trade price
    pub price: PxPrice,
    /// trade id
    // NOTE: in official docs it's stated as "Long", but WS sends String?!
    pub id: String,
    /// time the record was pushed
    #[serde(rename = "ts")]
    pub record_time: PxTimestamp,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TakerSide {
    Buy,
    Sell,
}
impl TakerSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

impl TradesMessage {
    pub fn recent_trade(&self) -> RecentTrade {
        let Self {
            symbol,
            amount,
            taker_side,
            create_time,
            price,
            id,
            ..
        } = self;
        let_clone!(symbol: pair, price, amount, id: tid);
        RecentTrade {
            tid,
            // TODO: make sure it looks like "BTC_USDT"
            pair,
            price,
            amount,
            side: taker_side.as_str().into(),
            // it's safe to convert u64 milliseconds to i64, it's still enough for ~242 million years
            // but it's better if we encapsulate it in newtype, and use proper type for inner record type.
            timestamp: *create_time as _,
        }
    }
}
