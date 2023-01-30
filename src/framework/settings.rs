use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use serenity::{model::prelude::GuildId, prelude::Context};
use thiserror::Error;
use typemap::{Key, ShareMap};

#[derive(Debug, Clone, Error)]
pub enum SettingsError {}

impl Key for SettingsError {
    type Value = SettingsError;
}

#[async_trait]
pub trait Setting: Send + Sync + 'static + Sized {
    type Value: FromStr + Default + ToString + Send + Sync + 'static + Clone;

    fn name(&self) -> &'static str;

    async fn get(&self, ctx: &Context, guild_id: GuildId) -> Self::Value {
        let value = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().unwrap();
            settings
                .get(&guild_id)
                .and_then(|map| map.get::<SettingKey<Self>>().cloned())
        };

        match value {
            Some(value) => value,
            None => {
                let value = Self::Value::default();
                self.set(ctx, guild_id, value.clone()).await.unwrap();
                value
            }
        }
    }

    async fn set(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        value: Self::Value,
    ) -> Result<(), SettingsError> {
        let mut data = ctx.data.write().await;
        let settings = data.get_mut::<Settings>().unwrap();
        let map = settings.entry(guild_id).or_insert_with(ShareMap::custom);
        map.insert::<SettingKey<Self>>(value);
        Ok(())
    }
}

struct Settings;

impl serenity::prelude::TypeMapKey for Settings {
    type Value = HashMap<GuildId, ShareMap>;
}

pub struct SettingKey<T: Setting>(T);

impl<T: Setting> Key for SettingKey<T> {
    type Value = T::Value;
}
