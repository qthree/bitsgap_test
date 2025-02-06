use anyhow::Context;

pub mod candles;
pub mod intervals;
pub mod protocol;
pub mod trades;

pub async fn public_ws() -> anyhow::Result<()> {
    let (stream, response) = tokio_tungstenite::connect_async("wss://ws.poloniex.com/ws/public")
        .await
        .context("connect to poloniex public WS")?;
    dbg!(stream, response);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_public_ws() {
        super::public_ws().await.unwrap();
    }
}
