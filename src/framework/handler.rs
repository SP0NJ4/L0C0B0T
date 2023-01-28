use serenity::{client::Context, model::channel::Message};

use super::command::{Command, DispatchResult};
use super::utils::handle_error;

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

    pub async fn dispatch(&self, ctx: &Context, msg: &Message) {
        for command in &self.commands {
            match command.dispatch(ctx, msg).await {
                DispatchResult::Handled => {
                    println!("Ran {} command", command.name());
                }
                DispatchResult::Ignored => continue,
                DispatchResult::Error(error_msg) => {
                    handle_error(ctx, msg, error_msg).await;
                    return;
                }
            }
        }
    }
}
