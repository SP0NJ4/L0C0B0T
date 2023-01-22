// Play control commands

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

#[command]
#[only_in(guilds)]
#[aliases("p")]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.rest();

    let guild = msg.guild(&ctx.cache).unwrap();
    let author = msg.author.id;
    let channel = guild
        .voice_states
        .get(&author)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or("You must be in a voice channel")?;

    let manager = songbird::get(ctx).await.unwrap().clone();

    let (handler_lock, success) = manager.join(guild.id, channel).await;

    if success.is_err() {
        return Err("Failed to join voice channel".into());
    }

    let mut handler = handler_lock.lock().await;
    let source = songbird::input::ytdl_search(&query).await.unwrap();

    handler.enqueue_source(source);

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("s")]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = manager.get(guild.id).unwrap();
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    if queue.is_empty() {
        return Err("Queue is empty".into());
    }

    queue.skip().unwrap();

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
