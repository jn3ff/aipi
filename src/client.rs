pub mod claude;

use std::{error::Error, fmt::Display};

use claude::{ClaudeRequest, ClaudeResponse};
use reqwest::{RequestBuilder, Response};
use secrecy::ExposeSecret;

use crate::{
    message::{Message, MessageBundle, MessageMetadata},
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
    message_history: Vec<MessageBundle>,
    config: ModelConfig,
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: ModelConfig) -> LlmClient {
        LlmClient {
            message_history: Vec::new(),
            config,
            client: reqwest::Client::new(),
        }
    }

    fn to_payload(&self, message: &Message) -> String {
        match self.config.model {
            Model::Claude(_) => {
                let req = ClaudeRequest {
                    client: self,
                    next: message,
                };
                serde_json::to_string(&req).expect("correct serialization impl'd")
            }
            _ => todo!(),
        }
    }

    async fn handle_response(
        &mut self,
        antecedent: MessageBundle,
        response: Response,
    ) -> Result<(), LlmClientError> {
        let content = response
            .text()
            .await
            .map_err(|e| LlmClientError::ExtractContent(e.to_string()))?;
        match self.config.model {
            Model::Claude(_) => {
                let response = serde_json::from_str::<ClaudeResponse>(content.as_str())
                    .map_err(|e| LlmClientError::ParseResponse(e.to_string()))?;
                let message = Message::from(response);
                let message_metadata = MessageMetadata::new(&self.config);

                // update history if repsonse handling is successful
                self.message_history.push(antecedent);
                self.message_history
                    .push(MessageBundle::new(message, message_metadata));
            }
            Model::ChatGpt(_) => todo!(),
        }
        Ok(())
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), LlmClientError> {
        let bundle = MessageBundle::new(message, MessageMetadata::new(&self.config));
        let payload = self.to_payload(&bundle.message);
        let response = self
            .client
            .post(self.config.model.to_target_url())
            .with_model_headers(&self.config)
            .body(payload)
            .send()
            .await
            .map_err(|e| LlmClientError::Request(e.to_string()))?;

        self.handle_response(bundle, response).await?;
        Ok(())
    }

    pub fn print_message_history(&self) {
        println!("{:?}", self.message_history);
    }
}

pub trait WithModelHeaders {
    fn with_model_headers(self, config: &ModelConfig) -> Self;
}

impl WithModelHeaders for RequestBuilder {
    fn with_model_headers(self, config: &ModelConfig) -> Self {
        match config.model {
            Model::Claude(_) => self
                .header("x-api-key", config.token.expose_secret())
                .header("anthropic-version", config.model.to_api_version())
                .header("content-type", "application/json"),
            Model::ChatGpt(_) => todo!(),
        }
    }
}
