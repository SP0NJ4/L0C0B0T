use serenity::framework::standard::macros::group;

mod responses;
mod channels;
mod play;
mod queue;

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
