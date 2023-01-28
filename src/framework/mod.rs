use lazy_static::lazy_static;
use serenity::{
    framework::{
        standard::{macros::hook, CommandError},
        StandardFramework,
    },
    model::prelude::Message,
    prelude::Context,
};

use crate::commands::{music::MUSIC_GROUP, testing::TESTING_GROUP};

pub mod command;
pub mod handler;
pub mod utils;

use self::handler::L0C0B0THandler;

#[hook]
async fn before(
    _ctx: &serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    cmd_name: &str,
) -> bool {
    println!("Running {} command", cmd_name);
    true
}

#[hook]
async fn after(
    _ctx: &serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    cmd_name: &str,
    cmd_result: Result<(), CommandError>,
) {
    println!("Finished running {} command", cmd_name);

    if let Err(why) = cmd_result {
        println!("Error running command: {:?}", why);

        if let Err(why) = _msg
            .reply(&_ctx.http, format!("⚠️ **Error**: {}", why))
            .await
        {
            println!("Error sending error message: {:?}", why);
        }
    }
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    L0C0B0T_HANDLER.dispatch(ctx, msg).await;
}

lazy_static! {
    pub static ref L0C0B0T_HANDLER: L0C0B0THandler = L0C0B0THandler::new();
}

pub fn create_framework() -> StandardFramework {
    StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .before(before)
        .after(after)
        .normal_message(normal_message)
        .group(&TESTING_GROUP)
        .group(&MUSIC_GROUP)
}
