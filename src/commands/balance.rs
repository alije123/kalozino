use ormlite::Model;
use poise::{CreateReply, serenity_prelude as serenity};

use crate::{Context, database::get_connection, Error, models::database::Player};
use crate::utils::create_player;

/// Узнай свой ебаный баланс
#[tracing::instrument]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    aliases("$"),
    help_text_fn = "help_text"
)]
pub async fn balance(
    ctx: Context<'_>,
    #[description = "Выбери чела если хош"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let mut db = get_connection().await;
    let user = user.as_ref().unwrap_or_else(|| ctx.author());
    if user.id != ctx.framework().bot_id && user.bot {
        ctx.send(
            CreateReply::default()
                .embed(
                    serenity::CreateEmbed::default()
                        .title("Ошибка")
                        .description("Ты чо еблан какой баланс у ботов")
                        .color(serenity::Color::RED),
                )
                .reply(true)
                .allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
        .await?;
        return Ok(());
    }
    let player = match Player::select()
        .where_bind("id = ?", user.id.get() as i64)
        .fetch_optional(&mut db)
        .await?
    {
        Some(x) => x,
        None => create_player(ctx, user.clone(), &mut db).await?,
    };

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default().title("Баланс вообщем такой".to_string())
            .description(format!("{:.2}", player.balance))
            .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            ).thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1185992875134177341/Frame_145.png?ex=6591a0f0&is=657f2bf0&hm=e71b3c75aaa10c5c40f1e994acbd7e8844a0e879e6c80e53f75d446308ab4e94&")
            .color(serenity::Color::MEIBE_PINK)
        ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
    ).await?;
    Ok(())
}

fn help_text() -> String {
    serenity::MessageBuilder::new()
        .push("Еблан, команда должна быть такая: ")
        .push_mono("(/|!)balance|$ @пользователь-хуёльзователь|айди (можно не указывать бля)")
        .build()
}
