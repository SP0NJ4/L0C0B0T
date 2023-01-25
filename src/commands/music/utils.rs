// Shared utility functions for the music commands

use std::{sync::Arc, time::Duration};

use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    model::{
        channel::Message,
        prelude::{ChannelId, GuildId, UserId},
    },
    prelude::{Context, Mutex, TypeMapKey},
};
use songbird::{
    input::{Input, Restartable},
    Call,
};

use super::errors::MusicCommandError;

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
        .ok_or(MusicCommandError::NoVoiceChannel)?;

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
) -> Result<Arc<Mutex<Call>>, MusicCommandError> {
    let manager = songbird::get(ctx).await.unwrap().clone();

    let (handler_lock, success) = manager.join(guild_id, channel_id).await;

    success
        .map(|_| handler_lock)
        .map_err(|_| MusicCommandError::FailedToJoinChannel)
}

/// Pauses the current song
///
/// ## Arguments
///
/// * `handler_lock` - The lock to the songbird handler
///
/// ## Returns
///
/// * `Ok(())` - The song was paused
/// * `Err(&str)` - The song was not paused
pub(super) async fn pause_song(handler_lock: Arc<Mutex<Call>>) -> Result<(), MusicCommandError> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err(MusicCommandError::NoSongPlaying);
    }

    handler
        .queue()
        .pause()
        .map_err(|_| MusicCommandError::Other("Error al pausar"))?;

    Ok(())
}

/// Resumes the current song
///
/// ## Arguments
///
/// * `handler_lock` - The lock to the songbird handler
///
/// ## Returns
///
/// * `Ok(())` - The song was resumed
/// * `Err(&str)` - The song was not resumed
pub(super) async fn resume_song(handler_lock: Arc<Mutex<Call>>) -> Result<(), MusicCommandError> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err(MusicCommandError::NoSongPlaying);
    }

    handler
        .queue()
        .resume()
        .map_err(|_| MusicCommandError::Other("Error al reanudar"))?;

    Ok(())
}

lazy_static! {
    static ref DURATION_REGEX: Regex =
        Regex::new(r"^(?:(?:([01]?\d|2[0-3]):)?([0-5]?\d):)?([0-5]?\d)$").unwrap();
}

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
pub(super) fn parse_duration(input: &str) -> Option<Duration> {
    // If the input is a number, it's a duration in seconds
    if let Ok(seconds) = input.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    // Otherwise, it's a duration in the format `hh:mm:ss`
    let captures = DURATION_REGEX.captures(input)?;

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

#[derive(Debug, Clone, Copy)]
pub(super) enum QueuePosition {
    Last,
    Index(usize),
}

pub(super) struct TrackRequester;

impl TypeMapKey for TrackRequester {
    type Value = UserId;
}

pub(super) struct TrackChannel;

impl TypeMapKey for TrackChannel {
    type Value = ChannelId;
}

/// Add a song to the queue in a given position
///
/// ## Arguments
///
/// * `requester` - The user who requested the song
/// * `handler_lock` - A lock to the songbird handler
/// * `source` - The song to add to the queue
/// * `position` - The position to add the song to
///
/// ## Returns
///
/// * `Ok(usize)` - The index of the song in the queue
/// * `Err(&str)` - The song was not added to the queue
pub(super) async fn insert_song(
    requester: UserId,
    channel: ChannelId,
    handler_lock: Arc<Mutex<Call>>,
    source: Input,
    position: QueuePosition,
) -> Result<usize, MusicCommandError> {
    let mut handler = handler_lock.lock().await;

    // Add the song to the queue
    let handle = handler.enqueue_source(source);

    // Add custom metadata to the song
    {
        let mut typemap = handle.typemap().write().await;

        typemap.insert::<TrackRequester>(requester);
        typemap.insert::<TrackChannel>(channel);
    }

    // Modify the queue if necessary
    let queue = handler.queue();

    match position {
        QueuePosition::Last => Ok(queue.len() - 1),
        QueuePosition::Index(index) => {
            let queue = handler.queue();

            if index >= queue.len() || index == 0 {
                return Err(MusicCommandError::InvalidQueueIndex);
            }

            queue.modify_queue(move |q| {
                let song = q.remove(q.len() - 1).unwrap();
                q.insert(index, song);
            });

            Ok(index)
        }
    }
}

/// Searches for a song in youtube
///
/// ## Arguments
///
/// * `query` - The query to search for
///
/// ## Returns
///
/// * `Ok(Input)` - The song was found
/// * `Err(MusicCommandError)` - The song was not found
pub(super) async fn search_song(query: &str) -> Result<Restartable, MusicCommandError> {
    Restartable::ytdl_search(query, true)
        .await
        .map_err(|_| MusicCommandError::FailedVideoSearch)
}
