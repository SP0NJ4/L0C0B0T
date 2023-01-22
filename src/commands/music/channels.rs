use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::Context,
};

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or("You must be in a voice channel")?;

    let manager = songbird::get(ctx).await.unwrap().clone();

    let (_, success) = manager.join(guild.id, channel).await;

    match success {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to join voice channel".into()),
    }
}

#[command]
#[only_in(guilds)]
#[aliases("dc", "disconnect", "disc")]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    manager
        .remove(guild.id)
        .await
        .map_err(|_| "Failed to leave voice channel".into())
}
