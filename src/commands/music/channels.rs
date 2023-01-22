// Channel

use std::sync::Arc;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        channel::Message,
        prelude::{ChannelId, GuildId},
    },
    prelude::{Context, Mutex},
};
use songbird::Call;

pub(super) async fn get_guild_channel<'a>(
    ctx: &Context,
    msg: &Message,
) -> Result<(GuildId, ChannelId), &'a str> {
    let guild = msg.guild(&ctx.cache).unwrap();

    let channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|vs| vs.channel_id)
        .ok_or("You must be in a voice channel")?;

    Ok((guild.id, channel))
}

pub(super) async fn join_channel<'a>(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<Arc<Mutex<Call>>, &'a str> {
    let manager = songbird::get(ctx).await.unwrap().clone();

    let (handler_lock, success) = manager.join(guild_id, channel_id).await;

    success
        .map(|_| handler_lock)
        .map_err(|_| "Failed to join voice channel".into())
}

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild, channel) = get_guild_channel(ctx, msg).await?;

    join_channel(ctx, guild, channel)
        .await
        .map(|_| ())
        .map_err(|e| e.into())
}

#[command]
#[only_in(guilds)]
#[aliases("dc", "disconnect", "disc")]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    manager
        .remove(guild.id)
        .await
        .map_err(|_| "Failed to leave voice channel".into())
}
