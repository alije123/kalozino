use ormlite::{types::Json, Model};
use poise::{serenity_prelude as serenity, CreateReply};
use serde_json::json;

use crate::{
    database::get_connection, models::config::Starboard, models::database::Config, Context, Error,
};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS | MANAGE_GUILD_EXPRESSIONS",
    required_bot_permissions = "MANAGE_CHANNELS",
    default_member_permissions = "MANAGE_CHANNELS | MOVE_MEMBERS | MANAGE_GUILD_EXPRESSIONS"
)]
pub async fn starboard_forward_channel(
    ctx: Context<'_>,
    #[channel_types("Text")] channel: serenity::Channel,
    #[description = "Порог реакций для пересыла"] reaction_threshold: Option<usize>,
) -> Result<(), Error> {
    let Some(guild_channel) = channel.clone().guild() else {
        return Ok(());
    };
    if guild_channel.kind != serenity::ChannelType::Text {
        ctx.send(
            CreateReply::default()
                .embed(
                    serenity::CreateEmbed::default()
                        .title("Fok u!")
                        .description("Это не текстовый ченел!")
                        .color(serenity::Color::RED),
                )
                .reply(true)
                .allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
        .await?;
        return Ok(());
    }
    if reaction_threshold < Some(2) {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Fok u!")
                    .description("Слишком мало поставил число!")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
    }

    let mut db = get_connection().await;
    let mut config = Config::select()
        .where_bind("key = ?", "starboard")
        .fetch_optional(&mut db)
        .await?
        .unwrap_or(Config {
            key: "starboard".to_string(),
            server_id: guild_channel.guild_id.get() as i64,
            data: Json(serde_json::Value::Null),
        });

    let channels_whitelist = if !config.data.is_null() {
        if let Ok(existing_config) = serde_json::from_value::<Starboard>(config.data.0) {
            existing_config.channels_whitelist
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let reactions_min = reaction_threshold.unwrap_or(3);

    let data = json!(Starboard {
        forward_channel_id: guild_channel.id.get(),
        channels_whitelist,
        reaction_threshold: reactions_min,
    });

    config.data = Json(data.clone());

    if let Err(_) = config.clone().insert(&mut db).await {
        config.update_all_fields(&mut db).await?;
    };

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title(format!(
                    "Теперь в канал {} будет присылаться кал",
                    guild_channel.name
                ))
                .description(format!(
                    "У которого на сообщении  минимум {} реакций",
                    reactions_min
                ))
                .color(serenity::Color::ORANGE),
        ),
    )
    .await?;

    Ok(())
}
