use ormlite::{types::Json, Model};
use poise::{
    serenity_prelude::{self as serenity},
    CreateReply,
};
use serde_json::json;

use crate::{
    database::get_connection,
    models::{config::CustomVoice, database::Config},
    Context, Error,
};

/// Задать кастомный голосовой канал
#[tracing::instrument]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS",
    default_member_permissions = "MANAGE_CHANNELS | MOVE_MEMBERS"
)]
pub async fn custom_voice_spawn(
    ctx: Context<'_>,
    #[channel_types("Voice")] channel: serenity::Channel,
) -> Result<(), Error> {
    let Some(guild_channel) = channel.clone().guild() else {
        return Ok(());
    };
    if guild_channel.kind != serenity::ChannelType::Voice {
        ctx.send(
            CreateReply::default()
                .embed(
                    serenity::CreateEmbed::default()
                        .title("Fok u!")
                        .description("Это не войс!")
                        .color(serenity::Color::RED),
                )
                .reply(true)
                .allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
        .await?;
        return Ok(());
    }

    let Some(category_id) = guild_channel.parent_id else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default().title("Uh oh!")
                    .description("Я не вижу в какой категории находится этот канал, наверное к ней у меня нет доступа")
                    .color(serenity::Color::RED)
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
            .await?;
        return Ok(());
    };

    let mut db = get_connection().await;
    let mut config = Config::select()
        .where_bind("key = ?", "custom_voice_category")
        .fetch_optional(&mut db)
        .await?
        .unwrap_or(Config {
            key: "custom_voice".to_string(),
            server_id: guild_channel.guild_id.get() as i64,
            data: Json(serde_json::Value::Null),
        });
    let data = json!(CustomVoice {
        category_id: category_id.get(),
        voice_channel_id: guild_channel.id.get()
    });
    config.data = Json(data.clone());

    if let Err(_) = config.clone().insert(&mut db).await {
        config.update_all_fields(&mut db).await?;
    };

    ctx.send(
        CreateReply::default()
            .embed(
                serenity::CreateEmbed::default()
                    .title("O'gay!")
                    .description(format!(
                        "Войс {} установлен как войс чат для спавна войсов",
                        guild_channel.name,
                    ))
                    .color(serenity::Color::ORANGE),
            )
            .reply(true)
            .allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
    )
    .await?;

    Ok(())
}
