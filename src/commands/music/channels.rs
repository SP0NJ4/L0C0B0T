// Channel

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, prelude::ChannelId},
    prelude::{Context, Mentionable},
};

use super::{errors::MusicCommandError, utils::get_handler_lock};

/////////////////////////
//      Commands       //
/////////////////////////

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, true).await?;

    let handler = handler_lock.lock().await;

    let channel = handler.current_channel().unwrap();
    let channel_mention = ChannelId(channel.0).mention();

    msg.channel_id
        .say(&ctx.http, format!("**Conectando a {channel_mention}...**"))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("dc", "disconnect", "disc")]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, false).await?;

    let mut handler = handler_lock.lock().await;

    handler
        .leave()
        .await
        .map_err(|_| MusicCommandError::Other("No pude salir del canal"))?;

    msg.channel_id.say(&ctx.http, "Chau 😔").await?;

    Ok(())
}
