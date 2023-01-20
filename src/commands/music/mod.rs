use serenity::framework::standard::macros::group;

mod play;

use play::*;

#[group]
#[commands(play)]
struct Music;
