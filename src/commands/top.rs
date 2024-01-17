use std::fmt::Write;

use futures::{stream, StreamExt};
use ormlite::{postgres::PgConnection, Model, Row};
use poise::{
    serenity_prelude::{self as serenity, Mentionable},
    CreateReply,
};

use crate::{database::get_connection, models::database::Player, Context, Error};

/// Топ лучших на свете людей
#[tracing::instrument]
#[poise::command(slash_command, prefix_command, guild_only, aliases("%"))]
pub async fn top(
    ctx: Context<'_>,
    #[max = 20]
    #[min = 1]
    #[description = "На странице показать"]
    on_page: Option<usize>,
) -> Result<(), Error> {
    let on_page = on_page.unwrap_or(10usize);
    if on_page > 20 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Fok u")
                    .description("Нельзя больше 20 на странице")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    }
    if on_page < 1 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Fok u")
                    .description("Нельзя меньше 1 на странице")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    }
    let mut db = get_connection().await;

    let len: i64 = ormlite::query("SELECT COUNT(*) FROM players WHERE balance > 0")
        .fetch_one(&mut db)
        .await?
        .try_get(0)?;
    let len: usize = len.try_into()?;

    let total_pages = (len + on_page - 1) / on_page;

    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let reply = {
        // Send the embed with the first page as content

        let create_reply = crate::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Топ по балансу")
                .description(get_page_str(&mut db, ctx, on_page, 0).await?)
                .color(serenity::Color::BLUE)
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Страница {}/{}",
                    1, total_pages
                ))),
        );

        if len > on_page {
            let components = serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(&prev_button_id)
                    .emoji('◀')
                    .style(serenity::ButtonStyle::Secondary),
                serenity::CreateButton::new(&next_button_id).emoji('▶'),
            ]);
            create_reply.components(vec![components])
        } else {
            create_reply
        }
    };

    ctx.send(reply).await?;

    // Loop through incoming interactions with the navigation buttons
    let mut current_page: usize = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= total_pages {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(total_pages - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        let mut embed = serenity::CreateInteractionResponseMessage::new().embed(
            serenity::CreateEmbed::new()
                .title("Топ по балансу")
                .description(get_page_str(&mut db, ctx, on_page, current_page).await?)
                .color(serenity::Color::BLUE)
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Страница {}/{}",
                    current_page + 1,
                    total_pages
                ))),
        );
        let mut buttons = vec![];
        if current_page == 0 {
            buttons.push(
                serenity::CreateButton::new(&prev_button_id)
                    .emoji('◀')
                    .style(serenity::ButtonStyle::Secondary),
            );
            buttons.push(serenity::CreateButton::new(&next_button_id).emoji('▶'));
        } else if current_page == total_pages - 1 {
            buttons.push(serenity::CreateButton::new(&prev_button_id).emoji('◀'));
            buttons.push(
                serenity::CreateButton::new(&next_button_id)
                    .emoji('▶')
                    .style(serenity::ButtonStyle::Secondary),
            );
        } else {
            buttons.push(serenity::CreateButton::new(&prev_button_id).emoji('◀'));
            buttons.push(serenity::CreateButton::new(&next_button_id).emoji('▶'));
        }
        if !buttons.is_empty() {
            let components = serenity::CreateActionRow::Buttons(buttons);
            embed = embed.components(vec![components]);
        }
        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(embed),
            )
            .await?;
    }
    Ok(())
}

pub async fn get_page_str(
    db: &mut PgConnection,
    ctx: Context<'_>,
    on_page: usize,
    page: usize,
) -> Result<String, Error> {
    let players = Player::select()
        .order_desc("balance")
        .limit(on_page)
        .offset(page * on_page)
        .fetch_all(db)
        .await
        .unwrap();

    let player_stream = stream::iter(players.iter().filter(|player| player.balance > 0.0));
    let page_str = player_stream
        .enumerate()
        .fold(String::new(), |mut output, (idx, player)| async move {
            let user = serenity::UserId::new(player.id as u64)
                .to_user(&ctx)
                .await
                .unwrap();
            let guild = ctx.guild().and_then(|guild| Some(guild.to_owned()));
            let mut user_name = user.global_name.clone().unwrap_or(user.name.clone());
            if let Some(guild) = guild {
                if let Ok(member) = guild.member(&ctx, user.id).await {
                    if let Some(nick) = member.nick.clone() {
                        user_name = nick;
                    }
                }
            }
            let mention = user.mention().to_string();
            let _ = writeln!(
                output,
                "{}. **{}** ({}) - {:.2}",
                page * on_page + idx + 1,
                user_name,
                mention,
                player.balance
            );
            output
        })
        .await;

    Ok(page_str)
}
