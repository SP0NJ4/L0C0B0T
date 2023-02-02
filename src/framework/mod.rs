use lazy_static::lazy_static;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};

use crate::commands::sube_baja::SUBE_BAJA_COMMAND;

pub mod commands;
pub mod handler;
pub mod settings;
pub mod utils;

use self::handler::L0C0B0THandler;

#[command]
#[only_in(guilds)]
async fn setting(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mode: String = args.single()?;
    let setting: String = args.single()?;

    let guild_id = msg.guild_id.unwrap();

    match mode.as_str() {
        "set" => {
            let value = args.rest();
            L0C0B0T_HANDLER
                .set_setting(ctx, guild_id, &setting, &value)
                .await?;

            msg.reply(ctx, format!("`{setting}` actualizado a `{value}`"))
                .await?;
        }
        "get" => {
            let value = L0C0B0T_HANDLER.get_setting(ctx, guild_id, &setting).await?;
            msg.reply(ctx, format!("Valor de `{}`: `{}`", setting, value))
                .await?;
        }
        _ => {
            return Err("Modo inválido: `set` o `get`".into());
        }
    }

    Ok(())
}

#[group]
#[commands(setting)]
struct General;

lazy_static! {
    pub static ref L0C0B0T_HANDLER: L0C0B0THandler =
        L0C0B0THandler::new().command(SUBE_BAJA_COMMAND);
}
