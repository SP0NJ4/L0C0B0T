// Playback control

use std::{sync::Arc, time::Duration};

use regex::Regex;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::{Context, Mutex},
};
use songbird::Call;

use super::{
    channels::{get_guild_channel, join_channel},
    errors::MusicCommandError,
    queue::{insert_song, QueuePosition},
    responses::{
        searching_response, song_added_embed, song_seeked_response, song_skipped_response,
    },
};

pub(super) async fn pause_song(handler_lock: Arc<Mutex<Call>>) -> Result<(), &'static str> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err("No hay canción tocando");
    }

    handler.queue().pause().map_err(|_| "Error al pausar")?;

    Ok(())
}

pub(super) async fn resume_song(handler_lock: Arc<Mutex<Call>>) -> Result<(), &'static str> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err("No hay canción tocando");
    }

    handler.queue().resume().map_err(|_| "Error al resumir")?;

    Ok(())
}

const DURATION_PATTERN: &str = r"^(?:(?:([01]?\d|2[0-3]):)?([0-5]?\d):)?([0-5]?\d)$";

/// Parses a string in the format `hh:mm:ss` or `sss` to a `Duration`
///
/// ## Arguments
///
/// * `input` - The string to parse
///
/// ## Returns
///
/// * `Some(Duration)` - The parsed duration
/// * `None` - The string was not in the correct format
fn parse_duration(input: &str) -> Option<Duration> {
    // If the input is a number, it's a duration in seconds
    if let Ok(seconds) = input.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    // Otherwise, it's a duration in the format `hh:mm:ss`
    let captures = Regex::new(DURATION_PATTERN).unwrap().captures(input)?;

    let mut result = Duration::new(0, 0);

    if let Some(hours) = captures.get(1) {
        let hours = hours.as_str().parse::<u64>().unwrap();

        result += Duration::from_secs(hours * 3600);
    }

    let minutes = captures.get(2)?.as_str().parse::<u64>().unwrap();

    result += Duration::from_secs(minutes * 60);

    let seconds = captures.get(3)?.as_str().parse::<u64>().unwrap();

    result += Duration::from_secs(seconds);

    Some(result)
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
        msg.reply(ctx, searching_response(query)).await?;

        let source = songbird::input::ytdl_search(&query)
            .await
            .map_err(|_| MusicCommandError::FailedVideoSearch)?;

        let handler_lock = join_channel(ctx, guild, channel).await?;

        let position = insert_song(
            msg.author.id,
            channel,
            handler_lock.clone(),
            source,
            QueuePosition::Last,
        )
        .await?;

        let embed = {
            let handler = handler_lock.lock().await;
            let queue = handler.queue().current_queue();

            song_added_embed(ctx, &queue, position).await
        };

        msg.channel_id
            .send_message(&ctx.http, |m| m.set_embed(embed))
            .await?;

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

    msg.reply(ctx, searching_response(query)).await?;

    let source = songbird::input::ytdl_search(&query)
        .await
        .map_err(|_| MusicCommandError::FailedVideoSearch)?;

    let handler_lock = join_channel(ctx, guild, channel).await?;

    let song_playing = {
        let handler = handler_lock.lock().await;
        handler.queue().current().is_some()
    };

    let position = if song_playing {
        // If there is a song playing, insert the new song at index 1
        QueuePosition::Index(1)
    } else {
        // If there is no song playing, copy the behavior of play
        QueuePosition::Last
    };

    let position = insert_song(
        msg.author.id,
        channel,
        handler_lock.clone(),
        source,
        position,
    )
    .await?;

    let embed = {
        let handler = handler_lock.lock().await;
        let queue = handler.queue().current_queue();

        song_added_embed(ctx, &queue, position).await
    };

    msg.channel_id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await?;

    Ok(())
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

    let track = queue.current().ok_or(MusicCommandError::NoSongPlaying)?;

    queue.skip().unwrap();

    msg.channel_id
        .say(&ctx.http, song_skipped_response(&track))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();

    pause_song(handler_lock).await?;

    msg.channel_id.say(&ctx.http, "⏸️ **Pausando...**").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();

    resume_song(handler_lock).await?;

    msg.channel_id.say(&ctx.http, "▶️ **Reanudando...**").await?;

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

#[command]
#[only_in(guilds)]
pub async fn seek(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    let track = queue.current().ok_or(MusicCommandError::NoSongPlaying)?;

    let arg = args.rest();

    let position = parse_duration(arg).ok_or(MusicCommandError::InvalidTime)?;

    track.seek_time(position).unwrap();

    msg.channel_id
        .say(&ctx.http, song_seeked_response(&track, position))
        .await?;

    Ok(())
}
