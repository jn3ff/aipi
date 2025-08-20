use std::{error::Error, fmt::Display};

use reqwest::{RequestBuilder, Response};
use secrecy::ExposeSecret;
use tracing::{debug, info};

use crate::{
    message::{
        Message, MessageBundle, MessageMetadata,
        serde::{ModelRequestWrapper, ModelResponseWrapper},
    },
    models::{Model, ModelConfig},
};

#[derive(Debug, Clone)]
pub enum LlmClientError {
    Request(String),
    ParseResponse(String),
    ExtractContent(String),
}

impl Display for LlmClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Error for LlmClientError {}

#[derive(Debug, Clone)]
pub struct LlmClient {
    pub message_history: Vec<MessageBundle>,
    pub config: ModelConfig,
    pub(crate) client: reqwest::Client,
}

// pubs
impl LlmClient {
    pub fn new(config: ModelConfig) -> LlmClient {
        match config.model {
            // openai doesn't have system at the message level, it comes as the first in the stream of messages
            Model::ChatGpt(_) => {
                let mut history: Vec<MessageBundle> = Vec::new();
                if let Some(sys_prompt) = config.system_prompt.as_ref() {
                    let system_message = MessageBundle::new(
                        Message::from_system(sys_prompt.clone()),
                        MessageMetadata::new(&config),
                    );
                    history.push(system_message);
                }
                LlmClient {
                    message_history: history,
                    config,
                    client: reqwest::Client::new(),
                }
            }
            // probably Gemini just slides up into this block
            Model::Claude(_) => LlmClient {
                message_history: Vec::new(),
                config,
                client: reqwest::Client::new(),
            },
            Model::Gemini(_) => todo!("gem"),
            #[cfg(feature = "dev-tools")]
            Model::None => panic!("dev tools only"),
        }
    }

    /// message with adding to client's message history (useful for multisequenced interactions)
    pub async fn send_chat_message(&mut self, message: Message) -> Result<(), LlmClientError> {
        let bundle = self.bundle_message(message);
        let response = self.send_message_bundle(&bundle).await?;
        let response_bundle = self.extract_response(response).await?;

        // update history if response handling is successful
        self.message_history.push(bundle);
        self.message_history.push(response_bundle);
        Ok(())
    }

    /// message without adding to client's message history (useful if you don't care about history)
    pub async fn send_adhoc_message(
        &mut self,
        message: Message,
    ) -> Result<MessageBundle, LlmClientError> {
        let bundle = self.bundle_message(message);
        let response = self.send_message_bundle(&bundle).await?;
        let response_bundle = self.extract_response(response).await?;
        Ok(response_bundle)
    }

    pub fn log_message_history(&self) {
        info!("Message history: {:?}", self.message_history);
    }
}

// private
impl LlmClient {
    fn bundle_message(&self, message: Message) -> MessageBundle {
        MessageBundle::new(message, MessageMetadata::new(&self.config))
    }

    async fn send_message_bundle(
        &mut self,
        bundle: &MessageBundle,
    ) -> Result<Response, LlmClientError> {
        let wrapped_request = ModelRequestWrapper::new(bundle, self);
        let payload = wrapped_request.to_payload();

        debug!("Payload being sent {payload:?}");

        let response = self
            .client
            .post(self.config.model.to_target_url())
            .with_model_headers(&self.config)
            .body(payload)
            .inspect(|rb| {
                debug!("Inspecting built request before sending: {rb:?}");
            })
            .send()
            .await
            .map_err(|e| LlmClientError::Request(e.to_string()))?;

        Ok(response)
    }
    async fn extract_response(
        &mut self,
        response: Response,
    ) -> Result<MessageBundle, LlmClientError> {
        debug!("Unwrapping response: {response:?}");
        let content = response
            .text()
            .await
            .map_err(|e| LlmClientError::ExtractContent(e.to_string()))?;

        debug!("Deserializing and converting response content: {content:?}");

        let wrapped_response = ModelResponseWrapper::parse_new(content, &self.config)
            .map_err(|e| LlmClientError::ParseResponse(e.to_string()))?;

        debug!("Wrapped response {wrapped_response:?}");
        // build message from parsed & wrapped response
        let message = Message::from(wrapped_response);
        let message_metadata = MessageMetadata::new(&self.config);
        Ok(MessageBundle::new(message, message_metadata))
    }
}

pub trait WithModelHeaders {
    fn with_model_headers(self, config: &ModelConfig) -> Self;
    fn inspect(self, f: fn(s: &Self) -> ()) -> Self;
}

impl WithModelHeaders for RequestBuilder {
    fn with_model_headers(self, config: &ModelConfig) -> Self {
        match config.model {
            Model::Claude(_) => self
                .header("x-api-key", config.token.expose_secret())
                .header("anthropic-version", config.model.to_api_version())
                .header("content-type", "application/json"),
            Model::ChatGpt(_) => self
                .bearer_auth(config.token.expose_secret())
                .header("content-type", "application/json"),
            Model::Gemini(_) => todo!("gem"),
            #[cfg(feature = "dev-tools")]
            Model::None => panic!("dev-tools only"),
        }
    }

    fn inspect(self, f: fn(s: &Self) -> ()) -> Self {
        f(&self);
        self
    }
}
