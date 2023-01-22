use serenity::framework::standard::macros::group;

mod channels;
mod queue;

use channels::*;
use queue::*;

#[group]
#[commands(play, skip, join, leave)]
struct Music;
