use core::fmt;
use std::collections::BTreeMap;

use anyhow::{Context as _, bail};
use bitsgap_poloniex::{
    context::PoloniexContext,
    ws::{
        candles::CandlesMessage,
        channels::Channel,
        protocol::{ClientMsg, ServerEvent, ServerMsg, ServerStream},
        public_ws,
        trades::TradesMessage,
    },
};

use crate::storage::{OneOrMany, Storage};

// TODO: move partially to poloniex crate
pub(crate) async fn dump_events(
    context: &PoloniexContext,
    storage: &Storage,
    total_limit: Option<usize>,
    channels: BTreeMap<String, Channel>,
    symbols: &[&str],
) -> anyhow::Result<()> {
    let mut client = public_ws()
        .await
        .context("connect to poloniex public WebSocket server")?;

    {
        let symbols: Vec<_> = symbols.iter().copied().map(String::from).collect();
        let channel = channels.keys().cloned().collect();
        client
            .send(ClientMsg::Subscribe { channel, symbols })
            .await
            .ok()
            .context("send subscribe")?;
    }
    let mut total_stream_messages = 0;
    loop {
        let msg = client
            .recv()
            .await
            .context("receive message from WS server")?;
        match msg {
            ServerMsg::Stream(ServerStream { channel, data }) => {
                let Some(ch) = channels.get(&channel) else {
                    log::error!("Unknown channel: {channel}");
                    continue;
                };
                // TODO: compare event's symbol with desired symbols, just in case
                match ch {
                    Channel::Candles(interval) => {
                        let klines: OneOrMany<_> = data
                            .into_events()
                            .map(|res| {
                                res.and_then(|msg: CandlesMessage| {
                                    msg.kline(*interval, context)
                                        .context("convert candle to kline")
                                })
                            })
                            .filter_map(log_err(&channel))
                            .collect();
                        log::info!("New klines: {klines:?}");
                        for kline in klines {
                            total_stream_messages += storage
                                .upsert_kline(kline)
                                .await
                                .context("save kline from stream to storage")?;
                        }
                    }
                    Channel::Trades => {
                        let recent_trades = data
                            .into_events()
                            .filter_map(log_err(&channel))
                            .map(|msg: TradesMessage| msg.recent_trade())
                            .collect();
                        log::info!("New recent trades: {recent_trades:?}");
                        total_stream_messages += storage
                            .insert_recent_trades(recent_trades)
                            .await
                            .context("save recent trades from stream to storage")?;
                    }
                }
            }
            ServerMsg::Event(ServerEvent::Error { message }) => {
                bail!("WS server sent error event: {message:?}");
            }
            ServerMsg::Event(event) => {
                log::info!("WS server sent event: {event:?}");
            }
            ServerMsg::Subscriptions { subscriptions } => {
                log::info!("WS server sent subscriptions: {subscriptions:?}");
            }
        }
        if matches!(total_limit, Some(total_limit) if total_stream_messages >= total_limit) {
            break;
        }
    }
    Ok(())
}

fn log_err<T>(context: &impl fmt::Display) -> impl '_ + Fn(anyhow::Result<T>) -> Option<T> {
    move |res| match res {
        Ok(ok) => Some(ok),
        Err(err) => {
            log::error!("[{context}] {err:#}");
            None
        }
    }
}
