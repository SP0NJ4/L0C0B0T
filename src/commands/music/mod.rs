use serenity::framework::standard::macros::group;

mod channels;
mod play;
mod queue;

use channels::*;
use play::*;
use queue::*;

#[group]
#[commands(play, skip, stop, queue, join, leave)]
struct Music;
