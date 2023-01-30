use lazy_static::lazy_static;
use serenity::{
    framework::{
        standard::{macros::hook, CommandError},
        StandardFramework,
    },
    model::prelude::Message,
    prelude::Context,
};

use crate::commands::sube_baja::SUBE_BAJA_COMMAND;
use crate::commands::{music::MUSIC_GROUP, testing::TESTING_GROUP};

pub mod commands;
pub mod handler;
pub mod utils;

use self::handler::L0C0B0THandler;
use self::utils::handle_error;

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

lazy_static! {
    pub static ref L0C0B0T_HANDLER: L0C0B0THandler =
        L0C0B0THandler::new().command(SUBE_BAJA_COMMAND);
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
