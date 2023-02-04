use l0c0b0t_macros::define_setting;
use serenity::{
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::prelude::{ChannelId, GuildId, Message},
    prelude::Context,
};

use crate::utils::OptionalChannel;

define_setting!(music_channel: OptionalChannel);

pub(super) async fn get_music_channel(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    MUSIC_CHANNEL_SETTING.get(ctx, guild_id).await.unwrap().0
}

#[check]
pub(super) async fn in_music_channel(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let guild_id = msg
        .guild_id
        .ok_or(Reason::User("No estás en un servidor".into()))?;
    let channel_id = msg.channel_id;

    let music_channel = get_music_channel(ctx, guild_id).await;

    if music_channel == None || music_channel == Some(channel_id) {
        Ok(())
    } else {
        Err(Reason::User("No estás en el canal de música".into()))
    }
}
