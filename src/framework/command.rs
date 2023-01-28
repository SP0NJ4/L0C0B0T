use async_trait::async_trait;
use serenity::{model::prelude::Message, prelude::Context};

/// Result of a command dispatch.
///
/// * `Handled` means that the command was handled and no other commands should be
/// tried.
///
/// * `Ignored` means that the command was not handled and other commands should be
/// tried.
///
/// * `Error` means that an error occurred while handling the command and no other
/// commands should be tried.
pub enum DispatchResult {
    Handled,
    Ignored,
    #[allow(dead_code)]
    Error(String),
}

#[async_trait]
pub trait Command: Sync + Send + 'static {
    fn name(&self) -> &'static str;

    async fn dispatch(&self, ctx: &Context, msg: &Message) -> DispatchResult;
}
