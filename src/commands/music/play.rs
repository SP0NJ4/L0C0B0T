// Playback control

use std::sync::Arc;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::{Context, Mutex},
};
use songbird::Call;

use super::{
    channels::{get_guild_channel, join_channel},
    queue::{insert_song, QueuePosition},
};

pub(super) async fn pause_song(handler_lock: Arc<Mutex<Call>>) -> Result<(), &'static str> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err("No song playing");
    }

    handler.queue().pause().map_err(|_| "Failed to pause")?;

    Ok(())
}

pub(super) async fn resume_song(handler_lock: Arc<Mutex<Call>>) -> Result<(), &'static str> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err("No song playing");
    }

    handler.queue().resume().map_err(|_| "Failed to resume")?;

    Ok(())
}

/////////////////////////
//      Commands       //
/////////////////////////

#[command]
#[only_in(guilds)]
#[aliases("p")]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (guild, channel) = get_guild_channel(ctx, msg).await?;

    let query = args.rest();

    if !query.is_empty() {
        // If there is a query, search for a video and play it
        let source = songbird::input::ytdl_search(&query)
            .await
            .map_err(|_| "Failed to find video")?;

        let handler_lock = join_channel(ctx, guild, channel).await?;

        insert_song(handler_lock, source, QueuePosition::Last).await?;

        Ok(())
    } else {
        // If there is no query, resume the current song
        let manager = songbird::get(ctx).await.unwrap().clone();

        let handler_lock = manager.get(guild).unwrap();

        resume_song(handler_lock).await.map_err(|e| e.into())
    }
}

#[command]
#[only_in(guilds)]
#[aliases("pete", "pt")]
pub async fn play_top(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (guild, channel) = get_guild_channel(ctx, msg).await?;

    let query = args.rest();

    if !query.is_empty() {
        let source = songbird::input::ytdl_search(&query)
            .await
            .map_err(|_| "Failed to find video")?;

        let handler_lock = join_channel(ctx, guild, channel).await?;

        let song_playing = {
            let handler = handler_lock.lock().await;
            handler.queue().current().is_some()
        };

        if song_playing {
            // If there is a song playing, insert the new song at index 1
            insert_song(handler_lock, source, QueuePosition::Index(1)).await?;
        } else {
            // If there is no song playing, copy the behavior of play
            insert_song(handler_lock, source, QueuePosition::Last).await?;
        }

        Ok(())
    } else {
        Err("No query provided".into())
    }
}

#[command]
#[only_in(guilds)]
#[aliases("s", "fs")]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err("No song playing".into());
    }

    queue.skip().unwrap();

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();

    pause_song(handler_lock).await.map_err(|e| e.into())
}

#[command]
#[only_in(guilds)]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();

    resume_song(handler_lock).await.map_err(|e| e.into())
}

#[command]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    manager.remove(guild.id).await.unwrap();

    Ok(())
}
