use serenity::{
    client::Context,
    model::{channel::Message, prelude::GuildId},
    prelude::TypeMapKey,
};

use super::{
    commands::traits::Command,
    settings::{Setting, SettingsError},
};

/// Handler for commands that are not called by prefix.
pub struct L0C0B0THandler {
    commands: Vec<Box<dyn Command>>,
    settings: Vec<Box<dyn Setting>>,
}

impl L0C0B0THandler {
    pub fn new() -> Self {
        Self {
            commands: vec![],
            settings: vec![],
        }
    }

    pub fn command(mut self, group: impl Command) -> Self {
        self.commands.push(Box::new(group));
        self
    }

    pub fn setting(mut self, setting: impl Setting) -> Self {
        self.settings.push(Box::new(setting));
        self
    }

    pub async fn set_setting(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        name: &str,
        value: &str,
    ) -> Result<(), SettingsError> {
        for setting in &self.settings {
            if setting.name() == name {
                setting.set_string(ctx, guild_id, value).await?;
                return Ok(());
            }
        }

        Err(SettingsError::InvalidSetting)
    }

    pub async fn get_setting(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        name: &str,
    ) -> Result<String, SettingsError> {
        for setting in &self.settings {
            if setting.name() == name {
                return setting.get_string(ctx, guild_id).await;
            }
        }

        Err(SettingsError::InvalidSetting)
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

pub struct HandlerRef {
    handler: &'static L0C0B0THandler,
}

impl HandlerRef {
    pub fn new(handler: &'static L0C0B0THandler) -> Self {
        Self { handler }
    }

    pub fn get(&self) -> &'static L0C0B0THandler {
        self.handler
    }
}

impl TypeMapKey for HandlerRef {
    type Value = Self;
}

pub async fn get_handler(ctx: &Context) -> &'static L0C0B0THandler {
    ctx.data
        .read()
        .await
        .get::<HandlerRef>()
        .expect("Handler not set")
        .get()
}
