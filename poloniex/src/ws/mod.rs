use std::time::Duration;

use anyhow::Context as _;
use bitsgap_shared::ws::{Message, SimpleJsonCodec, WsClient, WsConfig};
use protocol::{ClientMsg, ServerMsg};

pub mod candles;
pub mod intervals;
pub mod protocol;
pub mod trades;

pub async fn public_ws() -> anyhow::Result<WsClient<ClientMsg, ServerMsg>> {
    let ping = serde_json::to_string(&ClientMsg::Ping).context("ping message to json")?;
    let config = WsConfig {
        ping: Message::Text(ping.into()),
        // The WebSockets server expects a message or a ping every 30 seconds
        ping_interval: Duration::from_secs(20),
        uri: "wss://ws.poloniex.com/ws/public"
            .parse()
            .context("parse uri")?,
        codec: SimpleJsonCodec,
    };
    config.start().await
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use candles::CandlesMessage;
    use protocol::{ServerEvent, ServerStream};
    use trades::TradesMessage;

    use super::*;
    use crate::tests::PoloniexContext;

    async fn test_ws_public_channel<T: serde::de::DeserializeOwned + fmt::Debug>(
        ch: &str,
        num: usize,
    ) -> Vec<T> {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let mut client = public_ws().await.unwrap();
        let seconds = Duration::from_secs(1);
        assert!(client.try_recv().unwrap().is_none());

        client
            .send(ClientMsg::Subscribe {
                channel: vec![ch.into()],
                symbols: vec!["BTC_USDT".into()],
            })
            .await
            .unwrap();
        let msg = client.recv_timeout(seconds).await.unwrap().unwrap();
        assert_eq!(
            msg,
            ServerEvent::Subscribe { channel: ch.into() }.into_msg()
        );
        let mut messages = vec![];
        for _ in 0..num {
            let msg = client.recv().await.unwrap();
            match msg {
                ServerMsg::Stream(ServerStream { mut data, channel }) if channel == ch => {
                    assert_eq!(data.len(), 1);
                    println!("{data:?}");
                    let msg: T = serde_json::from_value(data.remove(0)).unwrap();
                    println!("{msg:?}");
                    messages.push(msg);
                }
                _ => unimplemented!(),
            }
        }
        messages
    }

    #[tokio::test]
    async fn test_public_ws_candles() {
        let ch = "candles_minute_1";
        let context = PoloniexContext::init();
        let messages = test_ws_public_channel::<CandlesMessage>(ch, 3).await;
        for msg in messages {
            let kline = msg.kline(ch, &context).unwrap();
            println!("{kline:?}");
        }
    }

    #[tokio::test]
    async fn test_public_ws_trades() {
        let ch = "trades";
        let messages = test_ws_public_channel::<TradesMessage>(ch, 3).await;
        for msg in messages {
            let recent_trade = msg.recent_trade();
            println!("{recent_trade:?}");
        }
    }
}
