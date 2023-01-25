// Queue functionality

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

use super::{
    errors::MusicCommandError,
    responses::{now_playing_embed, queue_embed, searching_response, song_added_embed},
    utils::{insert_song, search_song, QueuePosition},
};

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
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
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
    let index = args.single::<usize>()?;
    let query = args.rest();

    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();

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
        msg.channel_id,
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

    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if index >= queue.len() || index == 0 {
        return Err(MusicCommandError::InvalidQueueIndex.into());
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
        Err(MusicCommandError::EmptyQueue.into())
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
