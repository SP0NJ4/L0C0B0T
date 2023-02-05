use serenity::{
    framework::{
        standard::{
            macros::{group, hook},
            CommandError,
        },
        StandardFramework,
    },
    model::prelude::*,
    prelude::*,
};
use songbird::serenity::SerenityInit;

use crate::framework::{
    handler::HandlerRef,
    settings::{Settings, SETTING_COMMAND},
    utils::handle_error,
    L0C0B0T_HANDLER,
};

use crate::commands::{music::MUSIC_GROUP, testing::TESTING_GROUP};

#[group]
#[commands(setting)]
struct General;

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
        let intents = GatewayIntents::non_privileged()
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::GUILD_PRESENCES;

        let handler_ref = HandlerRef::new(&L0C0B0T_HANDLER);

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

        {
            let mut data = client.data.write().await;
            data.insert::<Settings>(Settings::try_load().unwrap_or(Settings::new()));
            data.insert::<HandlerRef>(handler_ref);
        }

        Ok(Self { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        println!("Starting client...");
        self.client.start().await
    }
}
