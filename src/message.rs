pub mod serde;

use std::{error::Error, fmt::Display};

use chrono::{TimeZone, Utc};

use crate::models::{ModelConfig, Role};

/// Mod purpose:
/// Define our local notion of a "message" as representing an exchange between two parties (AI/Human, AI/AI, AI/Something Else?)
///
///
/// Decision log:
/// 2025-08-13: Keeping metadata about the interaction (model, system prompt, temp, etc) in the container (dyn LlmClient)
/// rather than the message itself. We can revisit this decision if a need becomes apparent, but it felt like a cleaner use case.
/// Can always modify the config of a mut client to adjust targets mid-flight, and since we store messages in this local format
/// we can cleanly serde from any llm provider with a chat-global model param.
/// The negative of this decision is we lose some history functionality. We can gain this back in a handful of ways once a need is found for it
/// Essentially boiling down to parallel metadata record in a message bundle being kept as historical data against a client
///
/// 2025-08-15: decided to go with "MessageBundle" as the primary unit of transfer within the crate, with client exposing a clean interface to Message.
/// I'm starting to feel like I'm reinventing two wheels simultaneously, but I think it makes sense to have the client hold some persistent notion of config
/// so we can "swap" at the client level, with config at the message level used for historical reference only.

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn from_user(content: String) -> Message {
        Message {
            role: Role::User,
            content,
        }
    }

    pub fn from_ai(content: String) -> Message {
        Message {
            role: Role::Ai,
            content,
        }
    }

    pub fn from_system(content: String) -> Message {
        Message {
            role: Role::System,
            content,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageTimestamp(chrono::DateTime<Utc>);

impl MessageTimestamp {
    pub fn now() -> Self {
        MessageTimestamp(chrono::Utc::now())
    }
}

#[derive(Debug, Clone)]
pub struct MessageMetadata {
    timestamp: MessageTimestamp,
    config: ModelConfig,
}

// TODO-5: Metadata integrates with the notion of chat history simply, but not efficiently
// i.e. config metadata is likely shared across many message instances
// a more efficient representation would be to have a used_config bucket at the client level
// that metadata refers to.
// This is less obvious code, but if chat history gets very long,
// or this code is used in a highly parallel application
// we may prefer to implement it that way
impl MessageMetadata {
    pub fn new(config: &ModelConfig) -> Self {
        MessageMetadata {
            timestamp: MessageTimestamp::now(),
            config: config.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageBundle {
    pub metadata: MessageMetadata,
    pub message: Message,
}

impl MessageBundle {
    pub fn new(message: Message, metadata: MessageMetadata) -> Self {
        MessageBundle { message, metadata }
    }
}

#[derive(Debug, Clone)]
pub enum MessageError {
    Parse(String),
}

impl Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("self:?"))
    }
}

impl Error for MessageError {}
