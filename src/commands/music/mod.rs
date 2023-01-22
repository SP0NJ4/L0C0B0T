use serenity::framework::standard::macros::group;

mod channels;
mod play;

use channels::*;
use play::*;

#[group]
#[commands(play, join, leave)]
struct Music;
