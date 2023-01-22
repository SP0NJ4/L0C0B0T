// Play control

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

use super::{
    channels::{get_guild_channel, join_channel},
    queue::{insert_song, QueuePosition},
};

#[command]
#[only_in(guilds)]
#[aliases("p")]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (guild, channel) = get_guild_channel(ctx, msg).await?;

    let query = args.rest();

    if !query.is_empty() {
        let source = songbird::input::ytdl_search(&query)
            .await
            .map_err(|_| "Failed to find video")?;

        let handler_lock = join_channel(ctx, guild, channel).await?;

        insert_song(handler_lock, source, QueuePosition::Last).await?;

        Ok(())
    } else {
        Err("No query provided".into())
    }
}

#[command]
#[only_in(guilds)]
#[aliases("s")]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.is_empty() {
        return Err("Queue is empty".into());
    }

    queue.skip().unwrap();

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    manager.remove(guild.id).await.unwrap();

    Ok(())
}
