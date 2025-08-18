use crate::{
    client::LlmClient,
    message::{Message, MessageBundle, serde::MessageList},
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

/// Mod purpose:
/// Specifically implement the guts of a claude interaction according to anthropic's API spec

#[derive(Debug, Clone)]
pub(crate) struct ClaudeRequest<'a> {
    pub(crate) client: &'a LlmClient,
    pub(crate) next: &'a MessageBundle,
}

impl<'a> Serialize for ClaudeRequest<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct(
            "ClaudeRequest",
            4 + usize::from(self.client.config.system_prompt.is_some()),
        )?;

        st.serialize_field("model", &self.client.config.model.to_model_string())?;
        st.serialize_field("max_tokens", &self.client.config.max_tokens)?;
        st.serialize_field("temperature", &self.client.config.temperature)?;
        if let Some(sys) = &self.client.config.system_prompt {
            st.serialize_field("system", sys)?;
        }

        st.serialize_field(
            "messages",
            &MessageList {
                prev: &self.client.message_history,
                next: self.next,
                model: &self.client.config.model,
            },
        )?;

        st.end()
    }
}

impl Message {
    pub(crate) fn from_claude_response(mut value: ClaudeResponse) -> Self {
        let content = value
            .content
            .pop()
            .expect("gotta be something in here")
            .text;
        Message::from_ai(content)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ClaudeContent {
    pub(crate) r#type: String,
    pub(crate) text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ClaudeResponse {
    pub(crate) content: Vec<ClaudeContent>,
}
