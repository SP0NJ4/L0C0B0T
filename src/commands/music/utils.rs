// Shared utility functions for the music commands

use std::{sync::Arc, time::Duration};

use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    model::{
        channel::Message,
        prelude::{ChannelId, UserId},
    },
    prelude::{Context, Mutex, TypeMapKey},
};
use songbird::{
    input::{Input, Restartable},
    Call,
};

use super::errors::MusicCommandError;

/// Return a lock to the songbird handler, optionally joining the channel
/// if the bot is not in it
///
/// ## Arguments
///
/// * `ctx` - The context of the message
/// * `msg` - The message to get the guild and channel from
/// * `join_if_not_in_channel` - Whether to join the channel if the bot is not in it
///
/// ## Returns
///
/// * `Ok(Arc<Mutex<Call>>)` - The lock to the songbird handler
/// * `Err(&str)` - The bot failed to join the voice channel
pub(super) async fn get_handler_lock(
    ctx: &Context,
    msg: &Message,
    join_if_not_in_channel: bool,
) -> Result<Arc<Mutex<Call>>, MusicCommandError> {
    let manager = songbird::get(ctx).await.unwrap().clone();

    let guild = msg.guild(&ctx.cache).unwrap();

    let handler_lock = manager.get(guild.id);

    let handler_lock = if let Some(handler_lock) = handler_lock {
        handler_lock
    } else if !join_if_not_in_channel {
        return Err(MusicCommandError::NotInVoiceChannel);
    } else {
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|vs| vs.channel_id)
            .ok_or(MusicCommandError::NoVoiceChannel)?;

        let (handler_lock, success) = manager.join(guild.id, channel_id).await;

        success
            .ok()
            .map(|_| handler_lock)
            .ok_or(MusicCommandError::FailedToJoinChannel)?
    };

    Ok(handler_lock)
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

/// Stops the player
///
/// ## Arguments
///
/// * `handler_lock` - The lock to the songbird handler
///
/// ## Returns
///
/// * `Ok(())` - The player was stopped
/// * `Err(&str)` - No song was playing
pub(super) async fn stop_player(handler_lock: Arc<Mutex<Call>>) -> Result<(), MusicCommandError> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err(MusicCommandError::NoSongPlaying);
    }

    handler.queue().stop();

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
        typemap.insert::<TrackChannel>(ChannelId(handler.current_channel().unwrap().0));
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
