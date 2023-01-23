// Queue functionality

use std::sync::Arc;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::Message,
        prelude::{ChannelId, UserId},
    },
    prelude::{Context, Mutex, TypeMapKey},
};
use songbird::{input::Input, Call};

use super::responses::{now_playing_embed, queue_embed, searching_response, song_added_embed};

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
) -> Result<usize, &'static str> {
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
                return Err("Posición inválida");
            }

            queue.modify_queue(move |q| {
                let song = q.remove(q.len() - 1).unwrap();
                q.insert(index, song);
            });

            Ok(index)
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

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err("No estoy tocando nada".into());
    }

    let embed = queue_embed(ctx, &queue.current_queue()).await;

    msg.channel_id
        .send_message(&ctx, |m| m.set_embed(embed))
        .await?;

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

    let current_track = handler.queue().current();

    match current_track {
        Some(track) => {
            let embed = now_playing_embed(ctx, &track).await;

            msg.channel_id
                .send_message(&ctx.http, |m| m.set_embed(embed))
                .await?;

            Ok(())
        }
        None => Err("No hay canción".into()),
    }
}

#[command]
#[only_in(guilds)]
#[aliases("i")]
pub async fn insert(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let index = args.single::<usize>()?;
    let query = args.rest();

    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();

    msg.reply(ctx, searching_response(query)).await?;
    let source = songbird::input::ytdl_search(&query)
        .await
        .map_err(|_| "No encontré el video")?;

    let queue_length = {
        let handler = handler_lock.lock().await;
        handler.queue().len()
    };

    if index > queue_length || index == 0 {
        return Err("Posición inválida".into());
    }

    let position = insert_song(
        msg.author.id,
        msg.channel_id,
        handler_lock.clone(),
        source,
        QueuePosition::Index(index),
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
#[aliases("rm")]
pub async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let index = args.parse::<usize>().map_err(|_| "Índice inválido")?;

    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if index >= queue.len() || index == 0 {
        return Err("Posición inválida".into());
    }

    let mut removed_title: String = String::new();

    handler.queue().modify_queue(|q| {
        removed_title = q.remove(index).unwrap().metadata().title.clone().unwrap();
    });

    msg.channel_id
        .say(
            &ctx.http,
            format!("🗑️ **{removed_title}** fue eliminada de la cola"),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.len() <= 1 {
        Err("La cola está vacía".into())
    } else {
        handler.queue().modify_queue(|q| {
            q.drain(1..);
        });

        msg.channel_id
            .say(&ctx.http, "💥 **Limpiando la cola**")
            .await?;

        Ok(())
    }
}
