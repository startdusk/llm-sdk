use derive_builder::Builder;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::IntoRequest;

#[derive(Debug, Serialize, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateEmbeddingRequest {
    /// Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model (8192 tokens for text-embedding-ada-002), cannot be an empty string,
    #[builder(setter(into))]
    input: EmbeddingInput,

    /// ID of the model to use. You can use the List models API to see all of your available models
    #[builder(default)]
    model: EmbeddingModel,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    encoding_format: Option<EmbeddingEncodingFormat>,

    #[builder(default, setter(strip_option, into))]
    // setter(strip_option, into) 设置的时候去掉Option, into 就是如果传了 &str, 就自动执行它的into函数, 变成String
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    user: Option<String>,
}

// currently we don't support array of integers, or array of array of integers.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EmbeddingEncodingFormat {
    #[default]
    #[serde(rename = "float")]
    Float,

    #[serde(rename = "base64")]
    Base64,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
pub enum EmbeddingModel {
    #[default]
    #[serde(rename = "text-embedding-ada-002")]
    TextEmbeddingAda002,

    #[serde(rename = "text-embedding-ada-002-v2")]
    TextEmbeddingAda002V2,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEmbeddingResponse {
    pub object: EmbeddingObject,
    pub data: Vec<Embedding>,
    pub model: EmbeddingModel,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingUsage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Embedding {
    /// The index of the embedding in the list of embeddings.
    pub index: usize,

    /// The embedding vector, which is a list of floats. The length of vector depends on the model as listed in the embedding guide.
    pub embedding: Vec<f64>,

    /// The object type, which is always "embedding".
    pub object: EmbeddingObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingObject {
    Embedding,
    List,
}

impl CreateEmbeddingRequest {
    pub fn new(input: impl Into<EmbeddingInput>) -> Self {
        CreateEmbeddingRequestBuilder::default()
            .input(input)
            .build()
            .unwrap()
    }
}

impl From<String> for EmbeddingInput {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for EmbeddingInput {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(value: Vec<String>) -> Self {
        Self::StringArray(value)
    }
}

impl From<&[String]> for EmbeddingInput {
    fn from(value: &[String]) -> Self {
        Self::StringArray(value.to_vec())
    }
}

impl IntoRequest for CreateEmbeddingRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder {
        let url = format!("{base_url}/embeddings");
        client.post(url).json(&self)
    }
}

#[cfg(test)]
mod tests {
    use crate::SDK;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn string_create_embedding_should_work() -> Result<()> {
        let req = CreateEmbeddingRequest::new("The food was delicious and the waiter...");
        let res = SDK.create_embedding(req).await?;
        assert_eq!(res.object, EmbeddingObject::List);
        // response model id is different
        assert_eq!(res.model, EmbeddingModel::TextEmbeddingAda002V2);
        assert_eq!(res.data.len(), 1);
        let data = &res.data[0];
        assert_eq!(data.embedding.len(), 1536);
        assert_eq!(data.index, 0);
        Ok(())
    }

    #[tokio::test]
    async fn array_string_create_embedding_should_work() -> Result<()> {
        let req = CreateEmbeddingRequest::new(vec![
            "The food was delicious and the waiter...".into(),
            "Who i am?".into(),
        ]);
        let res = SDK.create_embedding(req).await?;
        assert_eq!(res.object, EmbeddingObject::List);
        // response model id is different
        assert_eq!(res.model, EmbeddingModel::TextEmbeddingAda002V2);
        assert_eq!(res.data.len(), 2);
        let data = &res.data[1];
        assert_eq!(data.embedding.len(), 1536);
        assert_eq!(data.index, 1);
        Ok(())
    }
}
