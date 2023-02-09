use lazy_static::lazy_static;
use serenity::prelude::TypeMapKey;

use crate::commands::{music::settings::MUSIC_CHANNEL_SETTING, sube_baja::SUBE_BAJA_COMMAND};

pub mod commands;
pub mod handler;
pub mod settings;
pub mod utils;

use self::handler::L0C0B0THandler;

lazy_static! {
    pub static ref L0C0B0T_HANDLER: L0C0B0THandler = L0C0B0THandler::new()
        .command(SUBE_BAJA_COMMAND)
        .setting(MUSIC_CHANNEL_SETTING);
}

impl TypeMapKey for L0C0B0T_HANDLER {
    type Value = &'static L0C0B0THandler;
}
