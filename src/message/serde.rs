use serde::{Serialize, Serializer, ser::SerializeSeq};

use crate::models::Model;

use super::Message;

// serialization help only
pub(crate) struct MessageList<'a> {
    pub(crate) prev: &'a [Message],
    pub(crate) message: &'a Message,
    pub(crate) model: &'a Model,
}
impl<'a> Serialize for MessageList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.prev.len() + 1))?;

        for m in self.prev {
            let role = &m.role.as_string(self.model);
            let out = select_out_message(self.model, role, &m.content);
            SerializeSeq::serialize_element(&mut seq, &out)?;
        }
        let role = &self.message.role.as_string(self.model);
        let out = select_out_message(self.model, role, &self.message.content);
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
    }
}
