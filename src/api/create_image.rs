use derive_builder::Builder;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::IntoRequest;

#[derive(Debug, Serialize, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct CreateImageRequest {
    /// A text description of the desired image(s). The maximum length is 1000 characters
    /// for dall-e-2 and 4000 characters for dall-e-3
    #[builder(setter(into))]
    prompt: String,

    /// The model to use for image generation.
    #[builder(default)]
    model: ImageModel,

    /// The number of images to generate. Must be between 1 and 10. For dall-e-3, only n=1 is supported.
    #[builder(default, setter(strip_option))] // setter(strip_option) 设置的时候去掉Option
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    n: Option<usize>,

    /// The quality of the image that will be generated. hd creates images with finer details
    /// and greater consistency across the image. This param is only supported for dall-e-3.
    #[builder(default, setter(strip_option))] // setter(strip_option) 设置的时候去掉Option
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    quality: Option<ImageQuality>,

    /// The format in which the generated images are returned. Must be one of url or b64_json.
    #[builder(default, setter(strip_option))] // setter(strip_option) 设置的时候去掉Option
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    response_format: Option<ImageResponseFormat>,

    /// The size of the generated images. Must be one of 256x256, 512x512, or 1024x1024 for dall-e-2.
    /// Must be one of 1024x1024, 1792x1024, or 1024x1792 for dall-e-3 models.
    #[builder(default, setter(strip_option))] // setter(strip_option) 设置的时候去掉Option
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    size: Option<ImageSize>,

    /// The style of the generated images. Must be one of vivid or natural. Vivid causes the model to
    /// lean towards generating hyper-real and dramatic images. Natural causes the model to produce more
    /// natural, less hyper-real looking images. This param is only supported for dall-e-3.
    #[builder(default, setter(strip_option))] // setter(strip_option) 设置的时候去掉Option
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    style: Option<ImageStyle>,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[builder(default, setter(strip_option, into))]
    // setter(strip_option, into) 设置的时候去掉Option, into 就是如果传了 &str, 就自动执行它的into函数, 变成String
    #[serde(skip_serializing_if = "Option::is_none")] // 如果为None, 序列化的时候就不序列化它
    user: Option<String>,
}

impl CreateImageRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        CreateImageRequestBuilder::default()
            .prompt(prompt)
            .build()
            .unwrap()
    }
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum ImageModel {
    #[default]
    #[serde(rename = "dall-e-3")]
    DallE3,
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageQuality {
    #[default]
    Standard,
    Hd,
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageResponseFormat {
    #[default]
    Url,
    B64Json,
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum ImageSize {
    #[default]
    #[serde(rename = "1024x1024")]
    Large,

    #[serde(rename = "1792x1024")]
    LargeWide,

    #[serde(rename = "1024x1792")]
    LargeTall,
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageStyle {
    #[default]
    Vivid,
    Natural,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateImageResponse {
    pub created: u64,
    pub data: Vec<ImageObject>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageObject {
    /// The base64-encoded JSON of the generated image, if response_format is b64_json.
    pub b64_json: Option<String>,

    /// The URL of the generated image, if response_format is url (default).
    pub url: Option<String>,

    /// The prompt that was used to generate the image, if there was any revision to the prompt.
    pub revised_prompt: String,
}

impl IntoRequest for CreateImageRequest {
    fn into_request(self, base_url: &str, client: Client) -> RequestBuilder {
        let url = format!("{base_url}/images/generations");
        client.post(url).json(&self)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::SDK;

    use super::*;
    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn create_image_request_should_serialize() -> Result<()> {
        let req = CreateImageRequest::new("draw a cute caterpillar");
        assert_eq!(
            serde_json::to_value(req)?,
            json!({
                "model": "dall-e-3",
                "prompt": "draw a cute caterpillar",
            })
        );
        Ok(())
    }

    #[test]
    fn create_image_request_custom_should_serialize() -> Result<()> {
        let req = CreateImageRequestBuilder::default()
            .prompt("draw a cute caterpillar")
            .style(ImageStyle::Natural)
            .quality(ImageQuality::Hd)
            .build()?;
        assert_eq!(
            serde_json::to_value(req)?,
            json!({
                "model": "dall-e-3",
                "prompt": "draw a cute caterpillar",
                "quality": "hd",
                 "style": "natural",
            })
        );
        Ok(())
    }

    #[ignore = "这个单元测试很贵, OpenAI生成一个图片就要4美分, 相当于人名币3毛钱"]
    #[tokio::test]
    async fn create_image_should_work() -> Result<()> {
        let req = CreateImageRequest::new("draw a cute caterpillar");
        let res = SDK.create_image(req).await?;
        assert_eq!(res.data.len(), 1);
        let image = &res.data[0];
        assert!(image.url.is_some());
        assert!(image.b64_json.is_some());
        println!("image: {:?}", image);
        fs::write(
            "/tmp/caterpillar.png",
            reqwest::get(image.url.as_ref().unwrap())
                .await?
                .bytes()
                .await?,
        )?;
        Ok(())
    }
}
