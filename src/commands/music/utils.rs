// Shared utility functions for the music commands

use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        prelude::{ChannelId, UserId},
    },
    prelude::{Context, Mutex, TypeMapKey},
};
use songbird::{
    id::GuildId,
    input::{Input, Restartable},
    tracks::PlayMode,
    Call, Event, EventContext, EventHandler, Songbird,
};

use super::errors::MusicCommandError;

lazy_static! {
    /// The time in seconds between each check for idle voice
    /// channels
    static ref IDLE_CHECK_PERIOD: u64 = env::var("IDLE_CHECK_PERIOD")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap();

    /// The maximum time in seconds a voice channel can be idle
    /// before being disconnected
    static ref IDLE_MAX_TIME: u64 = env::var("IDLE_TIME")
        .unwrap_or_else(|_| "180".to_string())
        .parse()
        .unwrap();
    static ref IDLE_MAX_COUNTS: u64 = *IDLE_MAX_TIME / *IDLE_CHECK_PERIOD;
}

struct IdleHandler {
    manager: Arc<Songbird>,
    guild_id: GuildId,
    count: Arc<AtomicU64>,
}

#[async_trait]
impl EventHandler for IdleHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track(track_list) = ctx else {
            return None;
        };

        let is_playing = track_list
            .iter()
            .any(|(state, _)| matches!(state.playing, PlayMode::Play));

        if is_playing {
            self.count.store(0, Ordering::Relaxed);
        } else {
            let count = self.count.fetch_add(1, Ordering::Relaxed);

            if count >= *IDLE_MAX_COUNTS {
                self.manager.remove(self.guild_id).await.ok()?;
            }
        }

        None
    }
}

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

        println!(
            "Joining voice channel {:?} in guild '{}'",
            channel_id, guild.name
        );

        let (handler_lock, success) = manager.join(guild.id, channel_id).await;

        if success.is_err() {
            return Err(MusicCommandError::FailedToJoinChannel);
        }

        {
            let mut handler = handler_lock.lock().await;

            handler.remove_all_global_events();

            handler.add_global_event(
                Event::Periodic(Duration::from_secs(*IDLE_CHECK_PERIOD), None),
                IdleHandler {
                    manager: manager.clone(),
                    guild_id: guild.id.into(),
                    count: Arc::new(AtomicU64::new(0)),
                },
            );
        }

        handler_lock
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
        .map_err(|_| MusicCommandError::Generic)?;

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
        .map_err(|_| MusicCommandError::Generic)?;

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

/// Removes a song from the queue
///
/// ## Arguments
///
/// * `handler_lock` - A lock to the songbird handler
/// * `index` - The index of the song to remove
///
/// ## Returns
///
/// * `Ok(String)` - The title of the song that was removed
/// * `Err` - The song was not removed
pub(super) async fn remove_song(
    handler_lock: Arc<Mutex<Call>>,
    index: usize,
) -> Result<String, MusicCommandError> {
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if index >= queue.len() || index == 0 {
        return Err(MusicCommandError::InvalidQueueIndex);
    }

    let mut removed_title = None;

    queue.modify_queue(|q| {
        removed_title = q.remove(index).unwrap().metadata().title.clone();
    });

    removed_title.ok_or(MusicCommandError::InvalidQueueIndex)
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
