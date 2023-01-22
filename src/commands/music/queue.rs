// Queue functionality

use std::sync::Arc;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::{Context, Mutex},
};
use songbird::{input::Input, Call};

#[derive(Debug, Clone, Copy)]
pub(super) enum QueuePosition {
    Last,
    Index(usize),
}

/// Add a song to the queue in a given position
///
/// ## Arguments
///
/// * `handler_lock` - A lock to the songbird handler
/// * `source` - The song to add to the queue
/// * `position` - The position to add the song to
///
/// ## Returns
///
/// * `Ok(())` - The song was added to the queue
/// * `Err(&str)` - The song was not added to the queue
pub(super) async fn insert_song<'a>(
    handler_lock: Arc<Mutex<Call>>,
    source: Input,
    position: QueuePosition,
) -> Result<(), &'a str> {
    let mut handler = handler_lock.lock().await;

    // Add the song to the queue
    handler.enqueue_source(source);

    // Modify the queue if necessary
    match position {
        QueuePosition::Last => Ok(()),
        QueuePosition::Index(index) => {
            let queue = handler.queue();

            if index >= queue.len() || index == 0 {
                return Err("Index out of bounds");
            }

            queue.modify_queue(move |q| {
                let song = q.remove(q.len() - 1).unwrap();
                q.insert(index, song);
            });

            Ok(())
        }
    }
}

/////////////////////////
//      Commands       //
/////////////////////////

#[command]
#[only_in(guilds)]
#[aliases("q")]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        return Err("Queue is empty".into());
    }

    let mut message: String = "Queue:\n".into();

    message += &queue
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let metadata = track.metadata();
            let title = metadata.title.as_ref().unwrap();

            format!("{i}. {title}")
        })
        .collect::<Vec<String>>()
        .join("\n");

    msg.channel_id.say(&ctx.http, message).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("nepe", "np")]
pub async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        return Err("Queue is empty".into());
    }

    let track = queue.first().unwrap();
    let metadata = track.metadata();
    let title = metadata.title.as_ref().unwrap();

    msg.channel_id
        .say(&ctx.http, format!("Now playing: {}", title))
        .await?;

    Ok(())
}
