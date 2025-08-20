use std::collections::HashMap;

use secrecy::ExposeSecret;
use serde::Deserialize;
use strum::IntoEnumIterator;
use tracing::error;

use crate::{client::WithModelHeaders, environment::get_api_key, models::ModelConfigBuilder};

use super::{ChatGptVersion, ClaudeVersion, GeminiVersion, Model};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref MAP: HashMap<&'static str, Model> = ModelMetadata::map_providers();
}

/// The purpose of this mod is to encapsulate utility functions for maintainers of this crate
/// it is therefore ugly as fuck and completely hacked together
/// as a result, might be better than my "clean" code
/// (I am starting to become disgusted with the layers of abstraction in this thing, need to collapse some of it)
pub async fn fetch_and_display_metadata() {
    check_claude().await;
    check_chatgpt().await;
}

async fn check_claude() {
    let config = ModelConfigBuilder::new(Model::Claude(ClaudeVersion::None))
        .build()
        .unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.anthropic.com/v1/models")
        .with_model_headers(&config)
        .send()
        .await
        .unwrap();

    let content = res.text().await.unwrap();
    let mut metadata = serde_json::from_str::<ModelList>(content.as_str()).unwrap();
    metadata.set_provider("Anthropic".to_string());
    metadata.display_unsupported_models();
}

async fn check_chatgpt() {
    let config = ModelConfigBuilder::new(Model::ChatGpt(ChatGptVersion::None))
        .build()
        .unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.openai.com/v1/models")
        .with_model_headers(&config)
        .send()
        .await
        .unwrap();

    let content = res.text().await.unwrap();
    let mut metadata = serde_json::from_str::<ModelList>(content.as_str()).unwrap();
    metadata.set_provider("OpenAI".to_string());
    metadata.display_unsupported_models();
}

fn demodel() -> Option<Model> {
    None
}
fn destring() -> Option<String> {
    None
}
fn debool() -> bool {
    false
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    #[serde(skip, default = "demodel")]
    pub internal_rep: Option<Model>, // need to normalize
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    data: Vec<ModelDescriptor>,
    #[serde(skip, default = "debool")]
    normalized: bool,
    #[serde(skip, default = "destring")]
    provider: Option<String>, // need to initialize
}

impl ModelList {
    pub fn set_provider(&mut self, provider: String) {
        self.provider = Some(provider)
    }

    pub fn normalize_representation(&mut self) {
        for d in &mut self.data {
            d.internal_rep = MAP.get(d.id.as_str()).cloned();
        }
    }

    pub fn display_unsupported_models(&mut self) {
        if !self.normalized {
            self.normalize_representation();
        }
        for d in self.data.as_slice() {
            if d.internal_rep.is_none() {
                error!(
                    "UNSUPPORTED MODEL {:?} FROM PROVIDER {:?}",
                    d.id,
                    self.provider
                        .as_ref()
                        .unwrap_or(&"[unspecified]".to_string())
                )
            }
        }
    }
}

pub enum ModelMetadata {
    List(Vec<ModelDescriptor>),
}

#[cfg(feature = "dev-tools")]
impl ModelMetadata {
    pub fn map_providers() -> HashMap<&'static str, Model> {
        let mut map: HashMap<&'static str, Model> = HashMap::new();
        for provider in Model::iter() {
            match provider {
                Model::Claude(_) => {
                    Self::insert_versions(&mut map, ClaudeVersion::iter(), Model::Claude)
                }
                Model::ChatGpt(_) => {
                    Self::insert_versions(&mut map, ChatGptVersion::iter(), Model::ChatGpt)
                }
                Model::Gemini(_) => {
                    Self::insert_versions(&mut map, GeminiVersion::iter(), Model::Gemini)
                }
                Model::None => (),
                _ => eprintln!("Add provider branch to maintenance map_providers"),
            }
        }
        map
    }

    fn insert_versions<T, F>(
        map: &mut HashMap<&'static str, Model>,
        iter: impl Iterator<Item = T>,
        f: F,
    ) where
        F: Fn(T) -> Model,
    {
        for version in iter {
            let complete = f(version);
            if let Some(model_str) = complete.to_model_string() {
                map.insert(model_str, complete);
            }
        }
    }
}
