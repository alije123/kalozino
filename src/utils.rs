use ormlite::{Model, postgres::PgConnection};
use poise::{CreateReply, serenity_prelude as serenity};

use crate::{Context, Error, models::database::Player};

pub async fn create_player(
    ctx: Context<'_>,
    discord_user: serenity::User,
    db: &mut PgConnection,
) -> Result<Player, Error> {
    let player = Player {
        id: discord_user.id.get() as i64,
        balance: 0.0,
        timely_last_at: None,
        timely_last_value: None,
        timely_end_at: None,
        last_steal_at: None,
    };
    player.clone().insert(db).await?;
    ctx.channel_id()
        .send_message(ctx.http(),
                      serenity::CreateMessage::default().embed(
                          serenity::CreateEmbed::default().title("Добро пожаловать в каловое казино")
                              .description("Здесь вы будете есть свой кал")
                              .color(serenity::Color::ORANGE)
                              .author(serenity::CreateEmbedAuthor::new(discord_user.name.clone())
                                  .icon_url(discord_user.face())
                              ).thumbnail(
                              "https://cdn.discordapp.com/attachments/1185987026365980712/1185987066027315280/sticker.webp?ex=65919b87&is=657f2687&hm=ea2ec90957ea0f355eaaa1c0ac669e6d5cf48def3ed1a345acbec6989d604770&"
                          )
                      ),
        )
        .await?;
    Ok(player)
}

pub async fn send_default_message_with_face(
    ctx: Context<'_>,
    embed: serenity::builder::CreateEmbed,
    user: &serenity::User,
) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .embed(
                embed.author(
                    serenity::CreateEmbedAuthor::new(user.name.clone())
                        .icon_url(user.face())
                        .url(format!("https://discord.com/users/{}", user.id)),
                ),
            )
            .reply(true)
            .allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
    )
        .await?;

    Ok(())
}
