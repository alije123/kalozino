use crate::models::database::Config;
use crate::{Context, Error};

use crate::database::get_connection;
use crate::models::config::Starboard;
use ormlite::types::Json;
use ormlite::Model;
use poise::{serenity_prelude as serenity, CreateReply};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS | MANAGE_GUILD_EXPRESSIONS",
    required_bot_permissions = "MANAGE_CHANNELS",
    default_member_permissions = "MANAGE_CHANNELS | MOVE_MEMBERS | MANAGE_GUILD_EXPRESSIONS"
)]
pub async fn starboard_whitelist(
    ctx: Context<'_>,
    #[channel_types("Text")] channels: Vec<serenity::Channel>,
) -> Result<(), Error> {
    let mut db = get_connection().await;
    let Some(mut config) = Config::select()
        .where_bind("key = ?", "starboard")
        .fetch_optional(&mut db)
        .await?
    else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ты чо!")
                    .description("Сначала настрой канал для старборда!")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    };
    let mut data: Starboard = serde_json::from_value(config.data.0)?;
    data.channels_whitelist = channels.iter().map(|x| x.id().get()).collect();
    config.data = Json(serde_json::to_value(data)?);
    config.update_all_fields(&mut db).await?;

    Ok(())
}
