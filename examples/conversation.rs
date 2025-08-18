use aipi::client::LlmClient;
use aipi::message::Message;
use aipi::models::{ChatGptVersion, Model, ModelConfigBuilder};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt as trace_fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing();
    let config = ModelConfigBuilder::new(Model::ChatGpt(ChatGptVersion::Gpt5))
        .with_max_tokens(1024)
        .with_temperature(1.0)
        .build()
        .expect("valid config & env");

    let mut client = LlmClient::new(config);

    let message = Message::from_user("Hi! what is your name?".to_string());
    client.send_message(message).await?;

    let follow_up_message = Message::from_user(
        "It's nice to meet you. Can you tell me something about something about something?"
            .to_string(),
    );
    client.send_message(follow_up_message).await?;

    let confirming_message = Message::from_user(
        "Thank you for that. Now can you please tell me everything we've just discussed?"
            .to_string(),
    );
    client.send_message(confirming_message).await?;

    client.log_message_history();

    Ok(())
}

fn setup_tracing() {
    let default_filter = EnvFilter::from_default_env();
    let json_layer = trace_fmt::layer().json().with_filter(default_filter);

    tracing_subscriber::registry().with(json_layer).init();
}
