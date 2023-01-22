// Queue commands

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::Context,
};

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
