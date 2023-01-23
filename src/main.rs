use std::env;

use dotenv::dotenv;

mod client;
mod commands;
mod framework;
mod globals;

use client::L0C0B0TClient;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file (if present)
    let _ = dotenv();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let mut client = L0C0B0TClient::new(&token)
        .await
        .map_err(|e| format!("Error creating client: {}", e))
        .unwrap();

    client.start().await.unwrap();
}
