use serenity::framework::standard::macros::group;

mod errors;
pub mod settings;
mod utils;

mod channels;
mod play;
mod queue;
mod responses;

use channels::*;
use play::*;
use queue::*;

use settings::IN_MUSIC_CHANNEL_CHECK;

#[group]
#[commands(
    play,
    play_top,
    skip,
    pause,
    stop,
    seek,
    queue,
    now_playing,
    insert,
    move_,
    remove,
    replace,
    clear,
    join,
    leave
)]
#[checks(in_music_channel)]
struct Music;
