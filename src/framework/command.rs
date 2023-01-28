use async_trait::async_trait;
use serenity::{model::prelude::Message, prelude::Context};

/// A non-prefix command.
#[async_trait]
pub trait Command: Sync + Send + 'static {
    fn name(&self) -> &'static str;

    /// Dispatches a message to the command.
    ///
    /// Returns `true` if the message was handled by the command, `false` otherwise.
    async fn dispatch(&self, ctx: &Context, msg: &Message) -> bool;
}
