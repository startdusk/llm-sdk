use derive_builder::Builder;
use reqwest::{Client, RequestBuilder};
use serde::Serialize;

use crate::IntoRequest;

#[derive(Debug, Serialize, Clone, Builder)]
pub struct SpeechRequest {
    /// One of the available TTS models: tts-1 or tts-1-hd
    #[builder(default)] // 设置默认值
    model: SpeechModel,

    /// The text to generate audio for. The maximum length is 4096 characters.
    #[builder(setter(into))]
    input: String,

    /// The voice to use when generating the audio. Supported voices are alloy, echo, fable, onyx, nova, and shimmer.
    #[builder(default)]
    voice: SpeechVoice,

    /// The format to audio in. Supported formats are mp3, opus, aac, and flac.
    #[builder(default)]
    response_format: SpeechResponseFormat,

    /// The speed of the generated audio. Select a value from 0.25 to 4.0. 1.0 is the default.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
}

#[derive(Debug, Serialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpeechResponseFormat {
    #[default]
    Mp3,
    Opus,
    Aac,
    Flac,
}

#[derive(Debug, Serialize, Clone, Copy, Default)]
pub enum SpeechModel {
    #[default]
    #[serde(rename = "tts-1")]
    Tts1,
    #[serde(rename = "tts-1-hd")]
    Tts1Hd,
}

#[derive(Debug, Serialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpeechVoice {
    Alloy,
    Echo,
    Fable,
    Onyx,
    #[default]
    Nova,
    Shimmer,
}

impl SpeechRequest {
    pub fn new(input: impl Into<String>) -> Self {
        SpeechRequestBuilder::default()
            .input(input)
            .build()
            .unwrap()
    }
}

impl IntoRequest for SpeechRequest {
    fn into_request(self, base_url: &str, client: Client) -> RequestBuilder {
        let url = format!("{base_url}/audio/speech");
        client.post(url).json(&self)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::SDK;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn speech_should_work() -> Result<()> {
        let req = SpeechRequest::new("The quick brown fox jumped over the lazy dog.");
        let res = SDK.speech(req).await?;

        fs::write("fixtures/test.mp3", res)?;
        Ok(())
    }

    #[tokio::test]
    async fn speech_should_work_chinese() -> Result<()> {
        let req = SpeechRequest::new("红领巾胸前挂, 祖国永远在心中。");
        let res = SDK.speech(req).await?;

        fs::write("fixtures/chinese.mp3", res)?;
        Ok(())
    }
}
