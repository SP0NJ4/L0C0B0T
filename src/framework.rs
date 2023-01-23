use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
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
#[hook]
async fn after(
    _ctx: &serenity::client::Context,
    _msg: &serenity::model::channel::Message,
    cmd_name: &str,
    cmd_result: Result<(), CommandError>,
) {
    println!("Finished running {} command", cmd_name);

    if let Err(why) = cmd_result {
        println!("Error running command: {:?}", why);

        if let Err(why) = _msg
            .channel_id
            .say(&_ctx.http, format!("⚠️ **Error**: {}", why))
            .await
        {
            println!("Error sending error message: {:?}", why);
        }
    }
}

// TODO: In the future, we'll replace StandardFramework with our own
// custom framework.
pub fn create_framework() -> StandardFramework {
    StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .before(before)
        .after(after)
        .group(&TESTING_GROUP)
        .group(&MUSIC_GROUP)
}
