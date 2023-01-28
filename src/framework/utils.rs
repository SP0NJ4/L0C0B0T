use serenity::{model::prelude::Message, prelude::Context};

pub async fn handle_error(ctx: &Context, msg: &Message, error: String) {
    msg.reply(&ctx, format!("⚠️ **Error**: {error}"))
        .await
        .unwrap();
}
