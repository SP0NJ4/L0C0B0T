use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufReader, BufWriter},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serenity::{
    framework::standard::macros::command,
    framework::standard::{Args, CommandResult},
    model::prelude::GuildId,
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};
use thiserror::Error;

use super::handler::get_handler;

#[derive(Debug, Clone, Error)]
pub enum SettingsError {
    #[error("Los settings no se pudieron acceder")]
    SettingsNotAccessible,

    #[error("El valor no es válido")]
    InvalidValue,

    #[error("Setting inválido")]
    InvalidSetting,
}

/// A setting for the bot.
/// This trait is used to define a setting's name, default value, and validation.
/// It also provides methods to get and set the setting's string value (`get_string` and `set_string`).
///
/// ---
///
/// The trait is type-agnostic, so all type-related logic must be handled using the `define_setting!` macro.
#[async_trait]
pub trait Setting: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn default_value(&self) -> String;

    fn validate(&self, s: &str) -> bool;

    /// Get the value of this setting for the given guild in a string format.
    ///
    /// If the value is not set, it will be set to the default value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value could not be set.
    async fn get_string(&self, ctx: &Context, guild_id: GuildId) -> Result<String, SettingsError> {
        let value = {
            let data = ctx.data.read().await;
            let settings = data
                .get::<Settings>()
                .ok_or(SettingsError::SettingsNotAccessible)?;
            settings.get(&guild_id, self.name())
        };

        match value {
            Some(value) => Ok(value),
            None => {
                let value = self.default_value();
                self.set_string(ctx, guild_id, &value).await?;
                Ok(value)
            }
        }
    }

    /// Set the value of this setting for the given guild.
    ///
    /// # Errors
    ///
    /// Returns an error if the value could not be set.
    async fn set_string(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        value: &str,
    ) -> Result<(), SettingsError> {
        if !self.validate(value) {
            return Err(SettingsError::InvalidValue);
        }

        let mut data = ctx.data.write().await;
        let settings = data
            .get_mut::<Settings>()
            .ok_or(SettingsError::SettingsNotAccessible)?;
        settings.set(&guild_id, self.name(), value);
        Ok(())
    }
}

/// The collection of settings for the bot.
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    settings: HashMap<u64, HashMap<String, String>>,
}

impl Settings {
    /// Create a new, empty settings collection.
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    /// Get the value of a setting for a guild.
    pub fn get(&self, id: &GuildId, setting: &str) -> Option<String> {
        self.settings
            .get(&id.0)
            .and_then(|map| map.get(setting).cloned())
    }

    /// Set the value of a setting for a guild.
    pub fn set(&mut self, id: &GuildId, setting: &str, value: &str) {
        let map = self.settings.entry(id.0).or_insert_with(HashMap::new);
        map.insert(setting.to_string(), value.to_string());
    }

    /// Try to load settings from the file specified in the `SETTINGS_PATH` environment variable.
    ///
    /// Returns `None` if the environment variable is not set or the file could not be opened.
    pub fn try_load() -> Option<Self> {
        let path = env::var("SETTINGS_PATH").ok()?;
        let file = File::open(&path).ok()?;
        let reader = BufReader::new(file);
        let settings = ron::de::from_reader(reader).ok()?;
        println!("Loaded settings from {path}");
        Some(settings)
    }

    /// Save the settings to the file specified in the `SETTINGS_PATH` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set or the file could not be opened.
    pub fn save(&self) -> Result<(), SettingsError> {
        let path = env::var("SETTINGS_PATH").map_err(|_| SettingsError::SettingsNotAccessible)?;
        let file = File::create(path).map_err(|_| SettingsError::SettingsNotAccessible)?;
        let mut writer = BufWriter::new(file);
        ron::ser::to_writer_pretty(&mut writer, self, ron::ser::PrettyConfig::default())
            .map_err(|_| SettingsError::SettingsNotAccessible)
    }
}

impl TypeMapKey for Settings {
    type Value = Self;
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_GUILD")]
async fn setting(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mode: String = args.single()?;
    let setting: String = args.single()?;

    let guild_id = msg.guild_id.unwrap();

    match mode.as_str() {
        "set" => {
            let value = args.rest();
            get_handler(ctx)
                .await
                .set_setting(ctx, guild_id, &setting, value)
                .await?;

            msg.reply(ctx, format!("`{setting}` actualizado a `{value}`"))
                .await?;
        }
        "get" => {
            let value = get_handler(ctx)
                .await
                .get_setting(ctx, guild_id, &setting)
                .await?;
            msg.reply(ctx, format!("Valor de `{setting}`: `{value}`"))
                .await?;
        }
        _ => {
            return Err("Modo inválido: `set` o `get`".into());
        }
    }

    Ok(())
}
