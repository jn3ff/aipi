pub mod serde;

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
    used_config: ModelConfig,
}

// TODO-5: Metadata integrates with the notion of chat history simply, but not efficiently
// i.e. config metadata is likely shared across many message instances
// a more efficient representation would be to have a used_config bucket at the client level
// that metadata refers to.
// This is less obvious code, but if chat history gets very long,
// or this code is used in a highly parallel application
// we may prefer to implement it that way
impl MessageMetadata {
    pub fn new(used_config: &ModelConfig) -> Self {
        MessageMetadata {
            timestamp: MessageTimestamp::now(),
            used_config: used_config.clone(),
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
