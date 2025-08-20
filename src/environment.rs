use std::{
    env::{self, VarError},
    fmt::Display,
    sync::{
        LazyLock, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use secrecy::SecretString;

use crate::models::Model;

/// Holds environment
static ENVIRONMENT: LazyLock<RwLock<AiPiEnvironment>> =
    LazyLock::new(|| RwLock::new(AiPiEnvironment::new()));
/// Determines if environment needs reloading
static RELOAD_NEEDED: AtomicBool = AtomicBool::new(false);

// TODO-4: make these overridable with a registry
pub const API_KEY_ANTHROPIC: &str = "API_KEY_ANTHROPIC";
pub const API_KEY_OPENAI: &str = "API_KEY_OPENAI";
pub const API_KEY_GOOGLE: &str = "API_KEY_GOOGLE";

#[derive(Clone, Debug)]
struct AiPiEnvironment {
    anthropic_key: Option<SecretString>,
    openai_key: Option<SecretString>,
    google_key: Option<SecretString>,
}

impl Display for AiPiEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // debug is safe bc secrets are held in SecretString
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl AiPiEnvironment {
    pub fn new() -> Self {
        // suppress error as .env is not the only place we can set our env, e.g. the actual environment
        let _ = dotenv::dotenv();
        let map_secret = |s: Result<String, VarError>| s.ok().map(SecretString::from);
        AiPiEnvironment {
            anthropic_key: map_secret(env::var(API_KEY_ANTHROPIC)),
            openai_key: map_secret(env::var(API_KEY_OPENAI)),
            google_key: map_secret(env::var(API_KEY_OPENAI)),
        }
    }
}

pub fn get_api_key(model: &Model) -> Result<SecretString, String> {
    if RELOAD_NEEDED.load(Ordering::Acquire)
        && RELOAD_NEEDED
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    {
        let mut wguard = ENVIRONMENT.write().expect("Guard poisoned");
        *wguard = AiPiEnvironment::new();
    }

    let rguard = ENVIRONMENT.read().expect("Guard poisoned");

    let (key, env_var) = match model {
        Model::Claude(_) => (&rguard.anthropic_key, API_KEY_ANTHROPIC),
        Model::ChatGpt(_) => (&rguard.openai_key, API_KEY_OPENAI),
        Model::Gemini(_) => (&rguard.google_key, API_KEY_GOOGLE),
        #[cfg(feature = "dev-tools")]
        _ => panic!("dev tools only"),
    };

    stateful_retrieve_key(key).map_err(|_| format!("Must set {env_var} in your env"))
}

fn stateful_retrieve_key(key: &Option<SecretString>) -> Result<SecretString, ()> {
    match key {
        None => {
            RELOAD_NEEDED.store(true, Ordering::SeqCst);
            Err(())
        }
        Some(k) => Ok(k.clone()),
    }
}
