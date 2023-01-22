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

/// Get the guild and channel the user is in
///
/// ## Arguments
///
/// * `ctx` - The context of the message
/// * `msg` - The message to get the guild and channel from
///
/// ## Returns
///
/// * `Ok((GuildId, ChannelId))` - The guild and channel the user is in
/// * `Err(&str)` - The user is not in a voice channel
pub(super) async fn get_guild_channel(
    ctx: &Context,
    msg: &Message,
) -> Result<(GuildId, ChannelId), &'static str> {
    let guild = msg.guild(&ctx.cache).unwrap();

    let channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|vs| vs.channel_id)
        .ok_or("You must be in a voice channel")?;

    Ok((guild.id, channel))
}

/// Join a voice channel
///
/// ## Arguments
///
/// * `ctx` - The context of the message
/// * `guild_id` - The guild containing the channel to join
/// * `channel_id` - The channel to join
///
/// ## Returns
///
/// * `Ok(Arc<Mutex<Call>>)` - The lock to the songbird handler
/// * `Err(&str)` - The bot failed to join the voice channel
pub(super) async fn join_channel(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<Arc<Mutex<Call>>, &'static str> {
    let manager = songbird::get(ctx).await.unwrap().clone();

    let (handler_lock, success) = manager.join(guild_id, channel_id).await;

    success
        .map(|_| handler_lock)
        .map_err(|_| "Failed to join voice channel".into())
}

/////////////////////////
//      Commands       //
/////////////////////////

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
