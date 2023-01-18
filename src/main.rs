use std::env;

use dotenv::dotenv;
use l0c0b0t::client::L0C0B0TClient;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let mut client = L0C0B0TClient::new(&token)
        .await
        .map_err(|e| format!("Error creating client: {}", e))
        .unwrap();

    client.start().await.unwrap();
}
