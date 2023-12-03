mod api;
pub use api::*;

use anyhow::{anyhow, Result};
use api::chat_completion::ChatCompletionResponse;
use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};

use bytes::Bytes;
use std::time::Duration;

use reqwest::{Client, RequestBuilder, Response};

const TIMEOUT: u64 = 30;

#[derive(Debug, Clone)]
pub struct LlmSdk {
    pub(crate) token: String,
    pub(crate) client: Client,
}

pub trait IntoRequest {
    fn into_request(self, client: Client) -> RequestBuilder;
}

impl LlmSdk {
    pub fn new(key: String) -> Self {
        Self {
            token: key,
            client: Client::new(),
        }
    }

    pub async fn chat_completion(
        &self,
        req: chat_completion::ChatCompletionRequest,
    ) -> Result<chat_completion::ChatCompletionResponse> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json::<ChatCompletionResponse>().await?)
    }

    pub async fn create_image(
        &self,
        req: create_image::CreateImageRequest,
    ) -> Result<create_image::CreateImageResponse> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json::<create_image::CreateImageResponse>().await?)
    }

    pub async fn speech(&self, req: speech::SpeechRequest) -> Result<Bytes> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.bytes().await?)
    }

    pub async fn whisper(
        &self,
        req: whisper::WhisperRequest,
    ) -> Result<whisper::WhisperResponse> {
        let is_json = req.is_json();
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        let ret = if is_json {
            res.json::<whisper::WhisperResponse>().await?
        } else {
            let text = res.text().await?;
            whisper::WhisperResponse { text }
        };
        Ok(ret)
    }

    fn prepare_request(&self, req: impl IntoRequest) -> RequestBuilder {
        let req = req.into_request(self.client.clone());
        let req = if self.token.is_empty() {
            req
        } else {
            req.bearer_auth(&self.token)
        };

        req.timeout(Duration::from_secs(TIMEOUT))
    }
}

#[async_trait]
trait SendAndLog {
    async fn send_and_log(self) -> Result<Response>;
}

#[async_trait]
impl SendAndLog for RequestBuilder {
    async fn send_and_log(self) -> Result<Response> {
        let res = self.send().await?;
        let status = res.status();
        if status.is_client_error() || status.is_server_error() {
            let text = res.text().await?;
            tracing::error!("API failed: {:#?}", text);
            return Err(anyhow!("API failed: {:#?}", text));
        }
        Ok(res)
    }
}

/// For tool function. If you have a function taht you want ChatGPT to call, you shall put
/// all params into a struct and derive schemars::JsonSchema for it. Then you can use
/// `YourStruct::to_schema()` to generate json schema for tools.
pub trait ToSchema: JsonSchema {
    fn to_schema() -> serde_json::Value;
}

impl<T: JsonSchema> ToSchema for T {
    fn to_schema() -> serde_json::Value {
        serde_json::to_value(schema_for!(Self)).unwrap()
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
}
