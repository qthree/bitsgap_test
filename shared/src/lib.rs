use std::{any::type_name, sync::Arc, time::Duration};

use anyhow::Context;
use reqwest::{Url, header::CONTENT_TYPE};
use serde::de::DeserializeOwned;
use utils::{
    time::SpanDuration,
    url::{BuildUrl, UrlBuilder},
};

pub mod auth;
pub mod interval;
pub mod records;
pub mod utils;
pub mod ws;

#[derive(Debug)]
pub struct ApiFactory {
    // it's recommended to build reqwest::Client once, not per each request, has to do with TLS initialization and connections pool
    // but can we use same reqwest client for different end-users of the service? Should be ok if we don't use reqwest::ClientBuilder::cookie_store or cookie_provider
    // this allows as to use only one reqwest client per app
    client: reqwest::Client,
    http_config: HttpConfig,
}

#[derive(Debug)]
pub struct ApiConfig {
    pub base_url: Url,
    pub auth: AuthMethod,
}

#[derive(Debug, Clone, clap::Args)]
pub struct HttpConfig {
    /// HTTP request to API fails if no bytes were read for this duration
    #[clap(long, default_value = "15s")]
    pub http_read_timeout: SpanDuration,
    /// how long to wait after failed HTTP request before next attempt
    #[clap(long, default_value = "5s")]
    pub http_retry_after: SpanDuration,
    /// HTTP request retry attempts to make
    #[clap(long, default_value_t = 3)]
    pub http_retry_attempts: u32,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            http_read_timeout: Duration::from_secs(15).into(),
            http_retry_after: Duration::from_secs(5).into(),
            http_retry_attempts: 5,
        }
    }
}

#[derive(Debug)]
pub enum AuthMethod {
    HmacSha256 { api_key: String, secret_key: String },
}

impl ApiFactory {
    pub fn init(http_config: HttpConfig) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .read_timeout(http_config.http_read_timeout.into())
                .build()
                .context("build http client")?,
            http_config,
        })
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
            http_config: self.http_config.clone(),
            //headers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiRequester<C> {
    config: Arc<ApiConfig>,
    client: reqwest::Client,
    context: C,
    http_config: HttpConfig,
    //headers: HeaderMap,
}

impl<C> ApiRequester<C> {
    pub fn context(&self) -> &C {
        &self.context
    }

    pub fn build_url<B: BuildUrl<C>>(&self, with: &B) -> anyhow::Result<Url> {
        UrlBuilder::build(&self.config.base_url, with, &self.context)
    }

    async fn send_request(
        &self,
        method: reqwest::Method,
        url: Url,
    ) -> anyhow::Result<reqwest::Response> {
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

    async fn get_string(&self, url: Url) -> anyhow::Result<String> {
        let res = self.send_request(reqwest::Method::GET, url).await?;
        // we don't use Response::json to separate different kinds of errors
        res.text().await.context("receive JSON response")
    }

    pub async fn get_json<T: DeserializeOwned, B: BuildUrl<C>>(
        &self,
        build_url: &B,
    ) -> anyhow::Result<T> {
        let url = self.build_url(build_url).context("build url")?;

        let mut attempts = self.http_config.http_retry_attempts;
        let retry_after = self.http_config.http_retry_after;
        let json = loop {
            match self.get_string(url.clone()).await {
                Ok(ok) => break ok,
                Err(err) if attempts > 0 => {
                    log::error!("HTTP GET {url:?}: {err:#}");
                    log::info!("{attempts} attempts left, waiting for {retry_after} before retry");
                    attempts -= 1;
                    continue;
                }
                Err(err) => return Err(err),
            }
        };
        let res = serde_json::from_str(&json);
        if res.is_err() {
            log::error!("Can't parse json as {}: {json}", type_name::<T>());
        }
        res.context("parse response as JSON")
    }

    pub async fn get_response<R: Request<Response: serde::de::DeserializeOwned> + BuildUrl<C>>(
        &self,
        build_url: &R,
    ) -> anyhow::Result<R::Response> {
        self.get_json(build_url).await
    }
}

pub trait Request {
    type Response;
}
