use serenity::{client::Context, model::channel::Message};

use super::command::Command;

/// Handler for commands that are not called by prefix.
pub struct L0C0B0THandler {
    commands: Vec<Box<dyn Command>>,
}

impl L0C0B0THandler {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn command(mut self, group: impl Command) -> Self {
        self.commands.push(Box::new(group));
        self
    }

    /// Dispatches a message to the commands.
    ///
    /// The message is dispatched to each command in order until one of them returns
    /// `true`.
    pub async fn dispatch(&self, ctx: &Context, msg: &Message) {
        for command in &self.commands {
            if command.dispatch(ctx, msg).await {
                println!("Ran {} command", command.name());
                return;
            }
        }
    }
}
