use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::IntoRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionRequest {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionResponse {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chioce {}

impl IntoRequest for ChatCompletionRequest {
    fn into_request(self, client: Client) -> RequestBuilder {
        client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&self)
    }
}
