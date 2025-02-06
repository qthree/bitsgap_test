use std::{marker::PhantomData, time::Duration};

use anyhow::{Context, bail};
use futures::{SinkExt, StreamExt};
use tokio::{
    sync::mpsc,
    time::{Instant, sleep_until, timeout},
};
pub use tokio_tungstenite::tungstenite::{Message, http::Uri};

use crate::utils::Strict;

pub struct WsConfig<C> {
    pub ping: Message,
    pub ping_interval: Duration,
    pub uri: Uri,
    pub codec: C,
}

pub struct WsClient<TX, RX> {
    connection: MpscDuplex<TX, RX>,
}
impl<TX, RX> WsClient<TX, RX> {
    pub async fn recv(&mut self) -> Option<RX> {
        self.connection.rx.recv().await
    }

    pub fn try_recv(&mut self) -> anyhow::Result<Option<RX>> {
        use mpsc::error::TryRecvError;
        match self.connection.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => bail!("chaneel closed"),
        }
    }

    pub async fn recv_timeout(&mut self, duration: Duration) -> anyhow::Result<Option<RX>> {
        timeout(duration, self.connection.rx.recv())
            .await
            .ok()
            .context("recv timeout")
    }

    pub async fn send(&mut self, msg: TX) -> Result<(), TX> {
        self.connection.tx.send(msg).await.map_err(|err| err.0)
    }
}

struct MpscDuplex<TX, RX> {
    rx: mpsc::Receiver<RX>,
    tx: mpsc::Sender<TX>,
}
impl<TX, RX> MpscDuplex<TX, RX> {
    fn pair(buffer: usize) -> (MpscDuplex<TX, RX>, MpscDuplex<RX, TX>) {
        let (tx, rx) = mpsc::channel(buffer);
        let (tx2, rx2) = mpsc::channel(buffer);
        (MpscDuplex { rx, tx: tx2 }, MpscDuplex { rx: rx2, tx })
    }
}

type WebSocketStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

struct WsHandler<C, S2C, C2S> {
    config: WsConfig<C>,
    user: MpscDuplex<S2C, C2S>,
    wss: WebSocketStream,
}

impl<C> WsConfig<C> {
    pub async fn start<C2S: Strict, S2C: Strict>(self) -> anyhow::Result<WsClient<C2S, S2C>>
    where
        C: Strict + CodecOut<C2S> + CodecIn<S2C>,
    {
        let (wss, _response) = tokio_tungstenite::connect_async(&self.uri)
            .await
            .context("connect to WebSocket server")?;
        let (connection, user) = MpscDuplex::pair(32);
        tokio::spawn(async move {
            let handler = WsHandler {
                config: self,
                user,
                wss,
            };
            if let Err(err) = handler.handle().await {
                log::error!("WS connection handler closed with error: {err:?}");
            } else {
                log::debug!("WS connection handler closed without error");
            }
        });
        Ok(WsClient { connection })
    }
}

// TODO: handle Pongs
impl<C, S2C, C2S> WsHandler<C, S2C, C2S>
where
    C: CodecOut<C2S> + CodecIn<S2C>,
{
    async fn handle(mut self) -> anyhow::Result<()> {
        let mut ping = None;
        loop {
            let ping_fut = sleep_until(
                *ping.get_or_insert_with(|| Instant::now() + self.config.ping_interval),
            );
            tokio::select!(
                biased;
                res = self.user.rx.recv() => {
                    if let Some(msg) = res {
                        if let Some(msg) = self.config.codec.process_out(msg).context("process internal message with codec")? {
                            self.wss.send(msg).await.context("send WebSocket message to server")?;
                            ping = None;
                        };
                    } else {
                        break;
                    }
                }
                _ = ping_fut => {
                    self.wss.send(self.config.ping.clone()).await.context("send ping message to server")?;
                    ping = None;
                }
                res = self.wss.next() => {
                    let Some(res) = res else {
                        break
                    };
                    let msg = res.context("recive WebSocket message from server")?;
                    let Some(msg) = self.config.codec.process_in(msg).context("process WebSocket message from server with codec")? else {
                        continue
                    };
                    if self.user.tx.send(msg).await.is_err() {
                        break
                    }
                },
            );
        }
        Ok(())
    }
}

pub trait CodecIn<MSG> {
    fn process_in(&mut self, msg: Message) -> anyhow::Result<Option<MSG>>;
}
pub trait CodecOut<MSG> {
    fn process_out(&mut self, msg: MSG) -> anyhow::Result<Option<Message>>;
}

#[derive(Default)]
pub struct SimpleJsonCodec;

impl<MSG: serde::Serialize> CodecOut<MSG> for SimpleJsonCodec {
    fn process_out(&mut self, msg: MSG) -> anyhow::Result<Option<Message>> {
        let msg = serde_json::to_string(&msg).context("serialize message as JSON")?;
        Ok(Some(Message::Text(msg.into())))
    }
}

impl<MSG: serde::de::DeserializeOwned> CodecIn<MSG> for SimpleJsonCodec {
    fn process_in(&mut self, msg: Message) -> anyhow::Result<Option<MSG>> {
        match msg {
            Message::Text(text) => {
                let res = serde_json::from_str(text.as_str());
                if res.is_err() {
                    log::error!("WebSocket server sent invalid JSON: {text:#?}");
                }
                return res.context("deserialize JSON from WebSocket text message");
            }
            Message::Close(close) => {
                log::debug!("WebSocket server sent WS close message: {close:?}")
            }
            msg => log::warn!("WebSocket server sent unsupported WS message: {msg:?}"),
        }
        Ok(None)
    }
}
