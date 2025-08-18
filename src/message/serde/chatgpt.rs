use crate::{
    client::LlmClient,
    message::{Message, MessageBundle, serde::MessageList},
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

/// Mod purpose:
/// Specifically implement the guts of a chatgpt interaction according to openAI's API spec

#[derive(Debug, Clone)]
pub(crate) struct ChatGptRequest<'a> {
    pub(crate) client: &'a LlmClient,
    pub(crate) next: &'a MessageBundle,
}

impl<'a> Serialize for ChatGptRequest<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("ChatGptRequest", 4)?;

        st.serialize_field("model", &self.client.config.model.to_model_string())?;
        st.serialize_field("max_completion_tokens", &self.client.config.max_tokens)?;
        st.serialize_field("temperature", &self.client.config.temperature)?;

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
    pub(crate) fn from_chatgpt_response(mut value: ChatGptResponse) -> Self {
        let content = value
            .choices
            .pop()
            .expect("gotta be something in here")
            .message
            .content;
        Message::from_ai(content)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ChatGptMessageContent {
    content: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ChatGptContent {
    pub(crate) index: usize, // currently unused, for multiplexing response
    pub(crate) message: ChatGptMessageContent,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ChatGptResponse {
    pub(crate) choices: Vec<ChatGptContent>,
}
