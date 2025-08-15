use client::LlmClient;
use message::Message;
use models::{ClaudeVersion, Model, ModelConfigBuilder};

mod client;
mod environment;
mod message;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ModelConfigBuilder::new(Model::Claude(ClaudeVersion::Sonnet4))
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

    let confirming_message = Message::from_user("Thank you for that. Now can you please tell me everything we've just discussed? every message... I know it's drab but I'm testing some software.".to_string());
    client.send_message(confirming_message).await?;

    client.print_message_history();

    Ok(())
}
