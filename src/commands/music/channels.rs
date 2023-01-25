// Channel

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::{Context, Mentionable},
};

use super::{
    errors::MusicCommandError,
    utils::{get_guild_channel, join_channel},
};

/////////////////////////
//      Commands       //
/////////////////////////

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild, channel) = get_guild_channel(ctx, msg).await?;

    join_channel(ctx, guild, channel).await.map(|_| ())?;

    let channel_mention = channel.mention();

    msg.channel_id
        .say(
            &ctx.http,
            format!("**Conectando a {}...**", channel_mention),
        )
        .await?;

    Ok(())
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
        .map_err(|_| MusicCommandError::NotInVoiceChannel)?;

    msg.channel_id.say(&ctx.http, "Chau 😔").await?;

    Ok(())
}
