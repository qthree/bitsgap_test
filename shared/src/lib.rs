use std::{any::type_name, sync::Arc};

use anyhow::Context;
use reqwest::{Url, header::CONTENT_TYPE};
use serde::de::DeserializeOwned;
use utils::url::{BuildUrl, UrlBuilder};

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
    pub fn context(&self) -> &C {
        &self.context
    }

    pub fn build_url<B: BuildUrl<C>>(&self, with: &B) -> anyhow::Result<Url> {
        UrlBuilder::build(&self.config.base_url, with, &self.context)
    }

    async fn send_request<B: BuildUrl<C>>(
        &self,
        method: reqwest::Method,
        build_url: &B,
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

    async fn get_string<B: BuildUrl<C>>(&self, build_url: &B) -> anyhow::Result<String> {
        let res = self.send_request(reqwest::Method::GET, build_url).await?;
        // we don't use Response::json to separate different kinds of errors
        res.text().await.context("receive JSON response")
    }

    pub async fn get_json<T: DeserializeOwned, B: BuildUrl<C>>(
        &self,
        build_url: &B,
    ) -> anyhow::Result<T> {
        let json = self.get_string(build_url).await?;
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
