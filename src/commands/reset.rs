use ormlite::Model;
use poise::{
    serenity_prelude::{self as serenity},
    CreateReply,
};

use crate::{database::get_connection, models::database::Player, Context, Error};

/// Сбросить всё к хуям
#[tracing::instrument]
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    let mut db = get_connection().await;
    let user = ctx.author();
    let Some(player) = Player::select()
        .where_bind("id = ?", user.id.get() as i64)
        .fetch_optional(&mut db)
        .await?
    else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ты уже стирал себе ASS")
                    .color(serenity::Color::RED),
            ).reply(true),
        )
        .await?;
        return Ok(());
    };
    player.delete(&mut db).await?;
    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Всё сброшено к хуям")
                .description("Можешь крутить себе таймли снова")
                .color(serenity::Color::RED)
                .author(serenity::CreateEmbedAuthor::new(user.name.clone()).icon_url(user.face())),
        ).reply(true),
    )
    .await?;

    Ok(())
}
