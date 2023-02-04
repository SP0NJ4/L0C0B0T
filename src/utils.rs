use std::{fmt::Display, str::FromStr};

use serenity::{
    model::prelude::{ChannelId, Mention},
    prelude::Mentionable,
};

pub struct OptionalChannel(pub Option<ChannelId>);

impl FromStr for OptionalChannel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self(None)),
            "ninguno" => Ok(Self(None)),
            _ => {
                let mention = Mention::from_str(s).map_err(|_| ())?;
                let channel_id = match mention {
                    Mention::Channel(guild_id) => guild_id,
                    _ => return Err(()),
                };

                Ok(Self(Some(channel_id)))
            }
        }
    }
}

impl Display for OptionalChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(channel_id) => write!(f, "{}", channel_id.mention()),
            None => write!(f, "ninguno"),
        }
    }
}

impl Default for OptionalChannel {
    fn default() -> Self {
        Self(None)
    }
}
