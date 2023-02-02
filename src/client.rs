use std::collections::HashMap;

use serenity::{
    framework::{
        standard::{macros::hook, CommandError},
        StandardFramework,
    },
    model::prelude::*,
    prelude::*,
};
use songbird::serenity::SerenityInit;

use crate::framework::{
    handler::L0C0B0THandler, settings::Settings, utils::handle_error, GENERAL_GROUP,
    L0C0B0T_HANDLER,
};

use crate::commands::{music::MUSIC_GROUP, testing::TESTING_GROUP};

#[hook]
async fn before(
    _ctx: &serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    cmd_name: &str,
) -> bool {
    println!("Running {cmd_name} command");
    true
}

#[hook]
async fn after(
    _ctx: &serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    cmd_name: &str,
    cmd_result: Result<(), CommandError>,
) {
    println!("Finished running {cmd_name} command");

    if let Err(why) = cmd_result {
        println!("Error running command: {why:?}");

        handle_error(_ctx, _msg, why.to_string()).await;
    }
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    L0C0B0T_HANDLER.dispatch(ctx, msg).await;
}

pub struct L0C0B0TClient {
    client: Client,
}

impl L0C0B0TClient {
    pub async fn new(token: &str) -> Result<Self, serenity::Error> {
        let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

        let client = serenity::Client::builder(token, intents)
            .framework(
                StandardFramework::new()
                    .configure(|c| c.prefix("!"))
                    .before(before)
                    .after(after)
                    .normal_message(normal_message)
                    .group(&TESTING_GROUP)
                    .group(&MUSIC_GROUP)
                    .group(&GENERAL_GROUP),
            )
            .register_songbird()
            .await?;

        client.data.write().await.insert::<Settings>(HashMap::new());

        Ok(Self { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        println!("Starting client...");
        self.client.start().await
    }
}
