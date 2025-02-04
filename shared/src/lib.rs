use anyhow::Context;
use reqwest::{IntoUrl, Url};
use serde::de::DeserializeOwned;

pub struct ApiRequester {
    // it's recommended to build reqwest::Client once, not per each request
    client: reqwest::Client,
    base_url: Url,
}

impl ApiRequester {
    pub fn new(base_url: impl IntoUrl) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: base_url.into_url().context("parse base url")?,
            client: reqwest::Client::new(),
        })
    }
    pub async fn get_string(&self, path: &str) -> anyhow::Result<String> {
        let url = self.base_url.join(path).context("join base url and path")?;
        let res = self.client.get(url).send().await.context("send GET request")?;
        res.text().await.context("receive JSON response")
    }
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let json = self.get_string(path).await?;
        serde_json::from_str(&json).context("parse response as JSON")
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_api_requester() {
        let requester = super::ApiRequester::new("https://api.poloniex.com").unwrap();
        let markets = requester.get_string("markets").await.unwrap();
        dbg!(markets);
    }
}
