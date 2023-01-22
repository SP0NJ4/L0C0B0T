use serenity::framework::standard::macros::hook;
use serenity::framework::StandardFramework;

use crate::commands::music::MUSIC_GROUP;
use crate::commands::testing::TESTING_GROUP;

#[hook]
async fn before(
    _ctx: &serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    _cmd_name: &str,
) -> bool {
    println!("Running {} command", _cmd_name);
    true
}

// TODO: In the future, we'll replace StandardFramework with our own
// custom framework.
pub fn create_framework() -> StandardFramework {
    StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .before(before)
        .group(&TESTING_GROUP)
        .group(&MUSIC_GROUP)
}