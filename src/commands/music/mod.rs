use serenity::framework::standard::macros::group;

mod errors;
mod utils;

mod channels;
mod play;
mod queue;
mod responses;

use channels::*;
use play::*;
use queue::*;

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
    remove,
    clear,
    join,
    leave
)]
struct Music;
