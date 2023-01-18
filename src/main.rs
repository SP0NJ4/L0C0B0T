// This is due to the package name: identifier names should be in snake case
#![allow(non_snake_case)]

use std::env;

use dotenv::dotenv;
use serenity::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .await
        .expect("Error creating client");
}
