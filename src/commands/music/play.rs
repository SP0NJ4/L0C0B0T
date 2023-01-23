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
    responses::{searching_response, song_added_embed, song_skipped_response},
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
            .map_err(|_| "No pude encontrar el video")?;

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

    if !query.is_empty() {
        msg.reply(ctx, searching_response(query)).await?;

        let source = songbird::input::ytdl_search(&query)
            .await
            .map_err(|_| "Failed to find video")?;

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
    } else {
        Err("No pusiste ninguna canción".into())
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

    match queue.current() {
        Some(track) => {
            queue.skip().unwrap();

            msg.channel_id
                .say(&ctx.http, song_skipped_response(&track))
                .await?;

            Ok(())
        }
        None => Err("No song playing".into()),
    }
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
