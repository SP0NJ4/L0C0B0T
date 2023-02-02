use std::collections::HashMap;

use async_trait::async_trait;
use serenity::{
    model::prelude::GuildId,
    prelude::{Context, TypeMapKey},
};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SettingsError {
    #[error("Los settings no se pudieron acceder")]
    SettingsNotAccessible,

    #[error("El valor no es válido")]
    InvalidValue,

    #[error("Setting inválido")]
    InvalidSetting,
}

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
            settings
                .get(&guild_id)
                .and_then(|map| map.get(self.name()).cloned())
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
        let map = settings.entry(guild_id).or_insert_with(HashMap::new);
        map.insert(self.name().to_string(), value.to_string());
        Ok(())
    }
}

pub struct Settings;

impl TypeMapKey for Settings {
    type Value = HashMap<GuildId, HashMap<String, String>>;
}
