use serenity::framework::standard::macros::group;

mod channels;
mod queue;

use channels::*;
use queue::*;

#[group]
#[commands(play, skip, stop, join, leave)]
struct Music;
