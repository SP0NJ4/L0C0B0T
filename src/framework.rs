use serenity::framework::StandardFramework;

pub fn create_framework() -> StandardFramework {
    StandardFramework::new().configure(|c| c.prefix(""))
}
