use std::sync::Arc;

use anyhow::Context;
use reqwest::{Url, header::CONTENT_TYPE};
use serde::de::DeserializeOwned;
use utils::url::{BuildUrl, UrlBuilder};

pub mod auth;
pub mod interval;
pub mod utils;

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
    HmacSha256 { api_key: String, secret_key: String },
}

impl ApiFactory {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn make_requester<C>(
        &self,
        config: impl Into<Arc<ApiConfig>>,
        context: C,
    ) -> ApiRequester<C> {
        //let headers = Default::default();
        ApiRequester {
            config: config.into(),
            client: self.client.clone(),
            context,
            //headers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiRequester<C> {
    config: Arc<ApiConfig>,
    client: reqwest::Client,
    context: C,
    //headers: HeaderMap,
}

impl<C> ApiRequester<C> {
    pub fn build_url(&self, with: impl BuildUrl<C>) -> anyhow::Result<Url> {
        UrlBuilder::build(&self.config.base_url, with, &self.context)
    }

    async fn send_request(
        &self,
        method: reqwest::Method,
        build_url: impl BuildUrl<C>,
    ) -> anyhow::Result<reqwest::Response> {
        let url = self.build_url(build_url).context("build url")?;

        let mut req = self
            .client
            .request(method, url)
            .header(CONTENT_TYPE, "application/json")
            .build()
            .context("build request")?;
        self.config
            .auth
            .apply(&mut req)
            .context("apply authentication to request")?;
        self.client.execute(req).await.context("execute request")
    }

    async fn get_string(&self, build_url: impl BuildUrl<C>) -> anyhow::Result<String> {
        let res = self.send_request(reqwest::Method::GET, build_url).await?;
        // we don't use Response::json to separate different kinds of errors
        res.text().await.context("receive JSON response")
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        build_url: impl BuildUrl<C>,
    ) -> anyhow::Result<T> {
        let json = self.get_string(build_url).await?;
        serde_json::from_str(&json).context("parse response as JSON")
    }
}
