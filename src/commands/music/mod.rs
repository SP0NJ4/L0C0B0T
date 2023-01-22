use serenity::framework::standard::macros::group;

mod channels;
mod play;
mod queue;

use channels::*;
use play::*;
use queue::*;

#[group]
#[commands(play, play_top, skip, stop, queue, now_playing, join, leave)]
struct Music;
