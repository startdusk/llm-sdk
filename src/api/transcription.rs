use derive_builder::Builder;
use reqwest::{
    multipart::{Form, Part},
    Client, RequestBuilder,
};
use serde::Deserialize;
use strum::{Display, EnumString};

use crate::IntoRequest;

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "mutable")]
pub struct TranscriptionRequest {
    /// The audio file object (not file name) to transcribe, in one of these formats: flac, mp3, mp4, mpeg, mpga, m4a, ogg, wav, or webm.
    file: Vec<u8>,

    /// ID of the model to use. Only whisper-1 is currently available.
    #[builder(default)]
    model: TranscriptionModel,

    /// The language of the input audio. Supplying the input language in ISO-639-1 format will improve accuracy and latency.
    #[builder(default, setter(strip_option, into))]
    language: Option<String>,

    /// An optional text to guide the model's style or continue a previous audio segment. The prompt should match the audio language.
    #[builder(default, setter(strip_option, into))]
    prompt: Option<String>,

    /// The format of the transcript output, in one of these options: json, text, srt, verbose_json, or vtt.
    #[builder(default)]
    response_format: TranscriptionResponseFormat,

    /// The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use log probability to automatically increase the temperature until certain thresholds are hit.
    #[builder(default, setter(strip_option, into))]
    temperature: Option<f32>,
}

#[derive(Debug, EnumString, Display, Clone, Copy, Default)]
#[strum(serialize_all = "snake_case")]
pub enum TranscriptionResponseFormat {
    #[default]
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

#[derive(Debug, EnumString, Display, Clone, Copy, Default)]
pub enum TranscriptionModel {
    #[default]
    #[strum(serialize = "whisper-1")]
    Whisper1,
}

#[derive(Debug, EnumString, Display, Clone, Copy, Default)]
pub enum SpeechModel {
    #[default]
    #[strum(serialize = "tts-1")]
    Tts1,
    #[strum(serialize = "tts-1-hd")]
    Tts1Hd,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TranscriptionResponse {
    pub text: String,
}

impl TranscriptionRequest {
    pub fn new(data: Vec<u8>) -> Self {
        TranscriptionRequestBuilder::default()
            .file(data)
            .build()
            .unwrap()
    }

    fn into_form(self) -> Form {
        let part = Part::bytes(self.file)
            .file_name("file")
            .mime_str("audio/mp3")
            .unwrap();
        let mut form = Form::new()
            .part("file", part)
            .text("model", self.model.to_string())
            .text("response_format", self.response_format.to_string());

        form = if let Some(language) = self.language {
            form.text("language", language)
        } else {
            form
        };
        form = if let Some(prompt) = self.prompt {
            form.text("prompt", prompt)
        } else {
            form
        };
        form = if let Some(temperature) = self.temperature {
            form.text("temperature", temperature.to_string())
        } else {
            form
        };

        form
    }
}

impl IntoRequest for TranscriptionRequest {
    fn into_request(self, client: Client) -> RequestBuilder {
        client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .multipart(self.into_form())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::LlmSdk;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn transctiption_should_work() -> Result<()> {
        let sdk = LlmSdk::new(std::env::var("OPENAI_API_KEY")?);
        let data = fs::read("fixtures/test.mp3")?;
        let req = TranscriptionRequest::new(data);
        let res = sdk.transcription(req).await?;
        assert_eq!(
            res.text.clone(),
            "The quick brown fox jumped over the lazy dog."
        );
        fs::write("fixtures/test.txt", res.text)?;
        Ok(())
    }
}
