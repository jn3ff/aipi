use std::{
    env,
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

#[derive(Clone, Debug)]
struct AiPiEnvironment {
    anthropic_key: Option<SecretString>,
    openai_key: Option<SecretString>,
}

impl Display for AiPiEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // debug is safe bc secrets are held in SecretString
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl AiPiEnvironment {
    pub fn new() -> Self {
        // suppress error as .env is not the only place we can load our env
        let _ = dotenv::dotenv();
        AiPiEnvironment {
            anthropic_key: env::var("API_KEY_ANTHROPIC").ok().map(SecretString::from),
            openai_key: env::var("API_KEY_OPENAI").ok().map(SecretString::from),
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

    match model {
        Model::Claude(_) => stateful_retrieve_key(&rguard.anthropic_key)
            .map_err(|_| String::from("Must set API_KEY_ANTHROPIC in your env")),
        Model::ChatGpt(_) => stateful_retrieve_key(&rguard.openai_key)
            .map_err(|_| String::from("Must set API_KEY_OPENAI in your env")),
    }
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
