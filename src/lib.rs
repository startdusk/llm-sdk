mod api;

pub use api::*;

use std::time::Duration;

use reqwest::{Client, RequestBuilder};

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
    ) -> anyhow::Result<chat_completion::ChatCompletionResponse> {
        let req = self.prepare_request(req);
        let res = req.send().await?;
        Ok(res
            .json::<chat_completion::ChatCompletionResponse>()
            .await?)
    }

    pub async fn create_image(
        &self,
        req: create_image::CreateImageRequest,
    ) -> anyhow::Result<create_image::CreateImageResponse> {
        let req = self.prepare_request(req);
        let res = req.send().await?;
        Ok(res.json::<create_image::CreateImageResponse>().await?)
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
