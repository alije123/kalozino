use crate::{Context, Error};

pub mod custom_voice_spawn;
pub mod starboard_forward_channel;
pub mod starboard_whitelist;

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    subcommand_required,
    subcommands(
        "custom_voice_spawn::custom_voice_spawn",
        "starboard_forward_channel::starboard_forward_channel",
        "starboard_whitelist::starboard_whitelist"
    )
)]
pub async fn set(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
