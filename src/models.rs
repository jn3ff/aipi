use std::{error::Error, fmt::Display};

use secrecy::SecretString;

use crate::environment::get_api_key;

/// Mod purpose:
/// Single enumeration place for all specific implementations by model
/// The goal is to add/extend support for any support by being able to only touch this file, add environment.rs, & add an llm/newthing for it.

#[derive(Debug, Clone, PartialEq)]
pub enum Model {
    Claude(ClaudeVersion),
    ChatGpt(ChatGptVersion),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClaudeVersion {
    Sonnet4,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatGptVersion {
    Gpt5,
}

impl Model {
    pub fn to_model_string(&self) -> &'static str {
        match self {
            Model::Claude(ver) => match ver {
                ClaudeVersion::Sonnet4 => "claude-sonnet-4-20250514",
            },
            Model::ChatGpt(ver) => match ver {
                ChatGptVersion::Gpt5 => "gpt-5",
            },
        }
    }

    // TODO-4: add "mode" so we can spec within-model
    pub fn to_target_url(&self) -> &'static str {
        match self {
            Model::Claude(_) => "https://api.anthropic.com/v1/messages",
            _ => todo!(),
        }
    }

    pub fn to_api_version(&self) -> &'static str {
        match self {
            Model::Claude(_) => "2023-06-01",
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    User,
    Ai,
    System,
}

impl Role {
    pub fn as_string(&self, model: &Model) -> String {
        match model {
            Model::Claude(_) => match self {
                Role::User => "user".to_string(),
                Role::Ai => "assistant".to_string(),
                Role::System => "system".to_string(), // TODO-2: this might not be correct in Claude spec and this function may need to return a result instead
            },
            Model::ChatGpt(ver) => match ver {
                ChatGptVersion::Gpt5 => match self {
                    Role::User => "user".to_string(),
                    Role::Ai => "assistant".to_string(),
                    Role::System => "developer".to_string(),
                },
            },
        }
    }
}

// TODO-5: Consider pub-ing fields at crate level
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model: Model,
    pub token: secrecy::SecretString,
    pub system_prompt: Option<String>,
    pub max_tokens: usize,
    pub temperature: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ModelConfigBuildError {
    NoTokenSet(String),
    Validation(String),
    Multi(Vec<ModelConfigBuildError>),
}

impl Display for ModelConfigBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Error for ModelConfigBuildError {}

pub struct ModelConfigBuilder {
    model: Model,
    system_prompt: Option<String>,
    max_tokens: Option<usize>,
    temperature: Option<f64>,
    errors: Vec<ModelConfigBuildError>,
}

impl ModelConfigBuilder {
    pub fn new(model: Model) -> ModelConfigBuilder {
        ModelConfigBuilder {
            model,
            system_prompt: None,
            max_tokens: None,
            temperature: None,
            errors: Vec::new(),
        }
    }

    pub fn with_system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        if !(0.0..=1.0).contains(&temperature) {
            self.errors.push(ModelConfigBuildError::Validation(format!(
                "Temperature parameter is out of bounds. Value supplied: {temperature}; Temperature bounds: [0, 1]."
            )));
        }
        self.temperature = Some(temperature);
        self
    }

    pub fn build(mut self) -> Result<ModelConfig, ModelConfigBuildError> {
        let token = match get_api_key(&self.model) {
            Ok(t) => t,
            Err(e) => {
                self.errors.push(ModelConfigBuildError::NoTokenSet(e));
                SecretString::from("")
            }
        };

        match self.errors.len() {
            0 => Ok(ModelConfig {
                model: self.model,
                token,
                system_prompt: self.system_prompt,
                max_tokens: self.max_tokens.unwrap_or(1024),
                temperature: self.temperature.unwrap_or(0.5),
            }),
            1 => Err(self.errors.pop().unwrap()),
            _ => Err(ModelConfigBuildError::Multi(self.errors)),
        }
    }
}
