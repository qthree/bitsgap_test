use std::{borrow::Cow, sync::Arc, time::Duration};

use anyhow::{bail, Context};
use reqwest::{header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE}, IntoUrl, RequestBuilder, Url};
use serde::de::DeserializeOwned;

mod auth;

#[derive(Debug)]
pub struct ApiFactory {
    // it's recommended to build reqwest::Client once, not per each request, has to do with TLS initialization and connections pool
    // but can we use same reqwest client for different end-users of the service? Should be ok if we don't use reqwest::ClientBuilder::cookie_store or cookie_provider
    // this allows as to use only one reqwest client per app
    client: reqwest::Client,
}

impl Default for ApiFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ApiConfig {
    pub base_url: Url,
    pub auth: AuthMethod,
}

#[derive(Debug)]
pub enum AuthMethod {
    HmacSha256{
        api_key: String,
        secret_key: String,
    }
}

impl ApiFactory {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    pub fn make_requester(&self, config: impl Into<Arc<ApiConfig>>) -> ApiRequester {
        //let headers = Default::default();
        ApiRequester {
            config: config.into(),
            client: self.client.clone(),
            //headers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiRequester {
    config: Arc<ApiConfig>,
    client: reqwest::Client,
    //headers: HeaderMap,
}

impl ApiRequester {
    async fn send_request(&self, method: reqwest::Method, path: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.config.base_url.join(path).context("join base url and path")?;

        let mut req = self.client.request(method, url)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .build()
            .context("build request")?;
        self.config.auth.apply(&mut req).context("apply authentication to request")?;
        self.client.execute(req).await.context("execute request")
    }
    async fn get_string(&self, path: &str) -> anyhow::Result<String> {
        let res = self.send_request(reqwest::Method::GET, path).await?;
        // we don't use Response::json to separate different kinds of errors
        res.text().await.context("receive JSON response")
    }
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let json = self.get_string(path).await?;
        serde_json::from_str(&json).context("parse response as JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_requester() {
        let api_key = std::env::var("API_KEY").expect("API_KEY env var");
        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY env var");
        let requester = ApiFactory::new().make_requester(ApiConfig{base_url: "https://api.poloniex.com".try_into().unwrap(), auth: AuthMethod::HmacSha256 { api_key, secret_key }});
        let markets: serde_json::Value = requester.get_json("markets").await.unwrap();
        println!("{}", serde_json::to_string_pretty(&markets).unwrap());
    }
}
