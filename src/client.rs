use serenity::prelude::*;

use crate::framework::create_framework;

pub struct L0C0B0TClient {
    client: Client,
}

impl L0C0B0TClient {
    pub async fn new(token: &str) -> Result<Self, serenity::Error> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let client = serenity::Client::builder(token, intents)
            .framework(create_framework())
            .await?;

        Ok(Self { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        println!("Starting client...");
        self.client.start().await
    }
}
