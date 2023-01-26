// Playback control

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

use super::{
    errors::MusicCommandError,
    responses::{
        searching_response, song_added_embed, song_seeked_response, song_skipped_response,
    },
    utils::{
        get_handler_lock, insert_song, parse_duration, pause_song, resume_song, search_song,
        stop_player, QueuePosition,
    },
};

#[command]
#[only_in(guilds)]
#[aliases("p")]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, true).await?;

    let query = args.rest();

    if !query.is_empty() {
        // If there is a query, search for a video and play it
        msg.reply(ctx, searching_response(query)).await?;
        let source = search_song(query).await?;

        let position = insert_song(
            msg.author.id,
            handler_lock.clone(),
            source.into(),
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
        resume_song(handler_lock).await.map_err(|e| e.into())
    }
}

#[command]
#[only_in(guilds)]
#[aliases("pete", "pt")]
pub async fn play_top(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, true).await?;

    let query = args.rest();

    msg.reply(ctx, searching_response(query)).await?;
    let source = search_song(query).await?;

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

    let position =
        insert_song(msg.author.id, handler_lock.clone(), source.into(), position).await?;

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
    let handler_lock = get_handler_lock(ctx, msg, false).await?;
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
    let handler_lock = get_handler_lock(ctx, msg, false).await?;

    pause_song(handler_lock).await?;

    msg.channel_id.say(&ctx.http, "⏸️ **Pausando...**").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, false).await?;

    resume_song(handler_lock).await?;

    msg.channel_id.say(&ctx.http, "▶️ **Reanudando...**").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, false).await?;

    stop_player(handler_lock).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn seek(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, false).await?;
    let handler = handler_lock.lock().await;

    let track = handler
        .queue()
        .current()
        .ok_or(MusicCommandError::NoSongPlaying)?;
    let duration = track.metadata().duration.unwrap();

    let arg = args.rest();
    let position = parse_duration(arg).ok_or(MusicCommandError::InvalidTime)?;

    if position > duration {
        return Err(MusicCommandError::InvalidTime.into());
    }

    track
        .seek_time(position)
        .map_err(|_| MusicCommandError::SeekFailed)?;

    msg.channel_id
        .say(&ctx.http, song_seeked_response(position))
        .await?;

    Ok(())
}
