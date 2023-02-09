// Queue functionality

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

use super::{
    errors::MusicCommandError,
    responses::{now_playing_embed, queue_embed, searching_response, song_added_embed},
    utils::{get_handler_lock, insert_song, remove_song, search_song, QueuePosition},
};

/////////////////////////
//      Commands       //
/////////////////////////

#[command]
#[only_in(guilds)]
#[aliases("q")]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, false).await?;
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.current().is_none() {
        return Err(MusicCommandError::NoSongPlaying.into());
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
    let handler_lock = get_handler_lock(ctx, msg, false).await?;
    let handler = handler_lock.lock().await;

    let track = handler
        .queue()
        .current()
        .ok_or(MusicCommandError::NoSongPlaying)?;

    let embed = now_playing_embed(ctx, &track).await;

    msg.channel_id
        .send_message(&ctx.http, |m| m.set_embed(embed))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("i")]
pub async fn insert(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, true).await?;

    let index = args.single::<usize>()?;
    let query = args.rest();

    msg.reply(ctx, searching_response(query)).await?;
    let source = search_song(query).await?;

    let queue_length = {
        let handler = handler_lock.lock().await;
        handler.queue().len()
    };

    if index > queue_length || index == 0 {
        return Err(MusicCommandError::InvalidQueueIndex.into());
    }

    let position = insert_song(
        msg.author.id,
        handler_lock.clone(),
        source.into(),
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

    let handler_lock = get_handler_lock(ctx, msg, false).await?;

    let removed_title = remove_song(handler_lock, index).await?;

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
#[aliases("re")]
pub async fn replace(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, true).await?;

    let query = args.rest();

    msg.reply(ctx, searching_response(query)).await?;
    let source = search_song(query).await?;

    let queue_length = {
        let handler = handler_lock.lock().await;
        handler.queue().len()
    };

    if queue_length <= 1 {
        return Err(MusicCommandError::EmptyQueue.into());
    }

    remove_song(handler_lock.clone(), queue_length - 1).await?;

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
}

#[command]
#[only_in(guilds)]
#[aliases("move", "mv")]
pub async fn move_(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let from = args
        .single::<usize>()
        .map_err(|_| MusicCommandError::InvalidQueueIndex)?;
    let to = args
        .single::<usize>()
        .map_err(|_| MusicCommandError::InvalidQueueIndex)?;

    let handler_lock = get_handler_lock(ctx, msg, false).await?;
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if from >= queue.len() || from == 0 {
        return Err(MusicCommandError::InvalidQueueIndex.into());
    }

    if to >= queue.len() || to == 0 {
        return Err(MusicCommandError::InvalidQueueIndex.into());
    }

    let mut moved_title: String = String::new();

    handler.queue().modify_queue(|q| {
        let track = q.remove(from).unwrap();
        moved_title = track.metadata().title.clone().unwrap();

        q.insert(to, track);
    });

    msg.channel_id
        .say(
            &ctx.http,
            format!("🚚 **{moved_title}** fue movida a la posición {to}"),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let handler_lock = get_handler_lock(ctx, msg, false).await?;
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.len() <= 1 {
        Err(MusicCommandError::EmptyQueue.into())
    } else {
        handler.queue().modify_queue(|q| {
            q.drain(1..);
        });

        msg.channel_id
            .say(&ctx.http, "💥 **Limpiando la cola...**")
            .await?;

        Ok(())
    }
}
