use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

#[command]
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

    handler.play_source(source);

    Ok(())
}
