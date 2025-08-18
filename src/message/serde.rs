use chatgpt::{ChatGptRequest, ChatGptResponse};
use claude::{ClaudeRequest, ClaudeResponse};
use serde::{Serialize, Serializer, ser::SerializeSeq};

mod chatgpt;
mod claude;

use crate::{
    client::LlmClient,
    models::{Model, ModelConfig},
};

use super::{Message, MessageBundle, MessageError};

pub trait ToMessage {
    fn to_message(&self) -> Message;
}

// serialization help only
pub(crate) struct MessageList<'a> {
    pub(crate) prev: &'a [MessageBundle],
    pub(crate) next: &'a MessageBundle,
    pub(crate) model: &'a Model,
}
impl<'a> Serialize for MessageList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.prev.len() + 1))?;

        for m in self.prev {
            let role = &m.message.role.as_string(self.model);
            let out = select_out_message(self.model, role, &m.message.content);
            SerializeSeq::serialize_element(&mut seq, &out)?;
        }
        let role = &self.next.message.role.as_string(self.model);
        let out = select_out_message(self.model, role, &self.next.message.content);
        SerializeSeq::serialize_element(&mut seq, &out)?;
        seq.end()
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum AnyOutMsg<'a> {
    Claude { role: &'a str, content: &'a str },
    ChatGpt { role: &'a str, content: &'a str },
}

fn select_out_message<'a>(model: &Model, role: &'a str, content: &'a str) -> AnyOutMsg<'a> {
    match model {
        Model::Claude(_) => AnyOutMsg::Claude { role, content },
        Model::ChatGpt(_) => AnyOutMsg::ChatGpt { role, content },
        _ => todo!("gem"),
    }
}

pub(crate) enum ModelRequestWrapper<'a> {
    Claude(ClaudeRequest<'a>),
    ChatGpt(ChatGptRequest<'a>),
    Gemini,
}

impl<'a> ModelRequestWrapper<'a> {
    pub(crate) fn new(next: &'a MessageBundle, client: &'a LlmClient) -> Self {
        match client.config.model {
            Model::Claude(_) => {
                let req = ClaudeRequest { next, client };
                ModelRequestWrapper::Claude(req)
            }
            Model::ChatGpt(_) => {
                let req = ChatGptRequest { next, client };
                ModelRequestWrapper::ChatGpt(req)
            }
            _ => todo!("gem"),
        }
    }

    // TODO-4 determine if this being panicable is acceptable given we should be able to verify correctness of serialization
    // w/in Rust's type system
    pub(crate) fn to_payload(&self) -> String {
        match self {
            Self::Claude(req) => serde_json::to_string(&req).expect("correct serialization impl'd"),
            Self::ChatGpt(req) => {
                serde_json::to_string(&req).expect("correct serialization impl'd")
            }
            Self::Gemini => todo!("gem"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ModelResponseWrapper {
    Claude(ClaudeResponse),
    ChatGpt(ChatGptResponse),
    Gemini,
}

impl From<ModelResponseWrapper> for Message {
    fn from(wrapper: ModelResponseWrapper) -> Self {
        match wrapper {
            ModelResponseWrapper::Claude(r) => Message::from_claude_response(r),
            ModelResponseWrapper::ChatGpt(r) => Message::from_chatgpt_response(r),
            ModelResponseWrapper::Gemini => todo!("gem"),
        }
    }
}

impl ModelResponseWrapper {
    pub fn parse_new(content: String, config: &'_ ModelConfig) -> Result<Self, MessageError> {
        let wrapped = match config.model {
            Model::Claude(_) => ModelResponseWrapper::Claude(
                serde_json::from_str::<ClaudeResponse>(content.as_str())
                    .map_err(|e| MessageError::Parse(e.to_string()))?,
            ),
            Model::ChatGpt(_) => ModelResponseWrapper::ChatGpt(
                serde_json::from_str::<ChatGptResponse>(content.as_str())
                    .map_err(|e| MessageError::Parse(e.to_string()))?,
            ),
            Model::Gemini(_) => todo!("gem"),
        };

        Ok(wrapped)
    }
}
