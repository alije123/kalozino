use chrono::{DateTime, Duration, Utc};
use ormlite::Model;
use poise::{serenity_prelude as serenity, CreateReply};
use rand::Rng;

use crate::{create_player, database::get_connection, models::database::Player, Context, Error};

#[tracing::instrument]
/// Получай по ебалу каждый день
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn timely(ctx: Context<'_>) -> Result<(), Error> {
    let mut db = get_connection().await;
    let user = ctx.author();
    let user_id = user.id.get() as i64;
    let mut player = match Player::select()
        .where_bind("id = ?", user_id)
        .fetch_optional(&mut db)
        .await?
    {
        Some(x) => x,
        None => create_player(ctx, user.clone(), &mut db).await?,
    };

    match (
        player.timely_last_at.clone(),
        player.timely_end_at.clone(),
        player.timely_last_value.clone(),
    ) {
        (None, None, None) => {
            tracing::info!("{}({}) gets first timely", user.name, user.id);
            first_time(ctx, user, &mut player).await?;
        }
        (Some(last_time), Some(end_at), Some(last_value)) => {
            let is_next_day = Utc::now() > last_time + Duration::days(1);
            let is_day_after_next_day = Utc::now() > last_time + Duration::days(2);
            let is_timely_end = Utc::now() > end_at;

            if is_next_day && !is_day_after_next_day && !is_timely_end {
                multiply_timely(ctx, user, &mut player, end_at, last_value).await?;
            } else if !is_next_day {
                let duration_to_next_day = (last_time + Duration::days(1) - Utc::now()) as Duration;
                let duration_str = format!(
                    "{} ч. {} мин.",
                    duration_to_next_day.num_hours(),
                    (duration_to_next_day - Duration::hours(duration_to_next_day.num_hours()))
                        .num_minutes()
                );

                send_fuck_you(ctx, user, duration_str).await?;
            } else if is_day_after_next_day {
                reset_timely(ctx, user, &mut player, ResetReason::SkippedDay).await?;
            } else if is_timely_end {
                reset_timely(ctx, user, &mut player, ResetReason::PeriodEnded).await?;
            }
        }
        _ => {
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title("Ой!")
                        .description("У тебя какой-то баганый профиль, напиши админам")
                        .color(serenity::Color::RED),
                ),
            )
            .await?;
            return Ok(());
        }
    };

    player.update_all_fields(&mut db).await?;

    Ok(())
}

#[tracing::instrument(name = "first_time")]
pub async fn first_time(
    ctx: Context<'_>,
    user: &serenity::User,
    player: &mut Player,
) -> Result<(), Error> {
    let (value, days, end) = generate_new_timely().await;
    player.timely_end_at = Some(end);
    player.timely_last_at = Some(Utc::now());
    player.timely_last_value = Some(value);
    player.balance += value;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default().title("Поздравляю с первый дейликом")
                .description(format!(
                    "На первый раз тебе начислено +{:.2}. Баланс теперь {:.2}. можешь засунуть их себе в ASS",
                    value, player.balance
                ))
                .color(serenity::Color::DARK_GREEN)
                .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                .thumbnail(
                    "https://cdn.discordapp.com/emojis/769582433229864972.webp?quality=lossless",
                )
                .footer(serenity::CreateEmbedFooter::new(format!(
                        "Каждый день в течение {} дней ты будешь получить на 0.2х больше. На следующий день ты получишь +{:.2}. Не забудь",
                        days, value * 1.2
                    ))
                )
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
    .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn multiply_timely(
    ctx: Context<'_>,
    user: &serenity::User,
    player: &mut Player,
    end_at: DateTime<Utc>,
    last_value: f64,
) -> Result<(), Error> {
    let value = last_value * 1.2;

    player.timely_last_at = Some(Utc::now());
    player.timely_last_value = Some(value);
    player.balance += value;

    let days_left = end_at.clone().signed_duration_since(Utc::now()).num_days();

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default().title("А вот и ещё один дейлик")
                .description(format!(
                    "Ты не проебался, поэтому держи: +{:.2}, на 0.2х больше, чем вчера",
                    value
                ))
                .footer(serenity::CreateEmbedFooter::new(format!("В течение ещё {} дней ты будешь получать каждый следующий день на 0.2 больше. На следующий день ты получишь +{:.2}. Не проебись!",
                        days_left,
                        value * 1.2
                    ))
                )
                .color(serenity::Color::PURPLE)
                .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                .thumbnail(
                    "https://cdn.discordapp.com/attachments/1185987026365980712/1186126903573217351/billy_04.png?ex=65921dc2&is=657fa8c2&hm=fcb564e1ff38e68b32d35c1df5351e84a26759581a883ae7b016efd19af630a3&"
                )
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
    .await?;

    Ok(())
}

#[tracing::instrument(name = "send_fuck_you", fields(user_id = %user.id.get(), duration_to_next_day = %duration_string))]
pub async fn send_fuck_you(
    ctx: Context<'_>,
    user: &serenity::User,
    duration_string: String,
) -> Result<(), Error> {
    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default().title("Иди нахуй 🖕")
                .description("Время ещё не тикнуло, день не прошёл")
                .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                .footer(serenity::CreateEmbedFooter::new(format!("Приходи через {}", duration_string)))
                .color(serenity::Color::RED)
                .thumbnail(
                    "https://cdn.discordapp.com/attachments/1185987026365980712/1186141895408222208/sticrgdfgfdgfdgker.webp?ex=65922bb9&is=657fb6b9&hm=5bd56e0829e49e48834ffe55e2b1ad7b167236a9b77349fb1b0f0d701f0034db&"
                )
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
    .await?;

    Ok(())
}

#[derive(Debug)]
pub enum ResetReason {
    SkippedDay,
    PeriodEnded,
}

#[tracing::instrument(name = "reset_timely", fields(user_id = %user.id.get()))]
pub async fn reset_timely(
    ctx: Context<'_>,
    user: &serenity::User,
    player: &mut Player,
    reason: ResetReason,
) -> Result<(), Error> {
    let (value, days, end) = generate_new_timely().await;

    player.timely_end_at = Some(end);
    player.timely_last_at = Some(Utc::now());
    player.balance += value;
    player.timely_last_value = Some(value);

    match reason {
        ResetReason::SkippedDay => {
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("Ты проебался")
                    .description(format!("Ты не получил дейлик вовремя, но всё равно держи: +{:.2}. Баланс теперь {:.2}", value, player.balance))
                .footer(serenity::CreateEmbedFooter::new(format!("Не проебись завтра, тебе капнет +{:.2}", value * 1.2)))
                .color(serenity::Color::RED)
                .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
            .thumbnail(
                "https://cdn.discordapp.com/attachments/1185987026365980712/1186141895408222208/sticrgdfgfdgfdgker.webp?ex=65922bb9&is=657fb6b9&hm=5bd56e0829e49e48834ffe55e2b1ad7b167236a9b77349fb1b0f0d701f0034db&"
            )
        ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
    ).await?;
        }
        ResetReason::PeriodEnded => {
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("Це кінець...")
                    .description(format!("Кажись период дейлика кончился, начинаем заново, даю тебе +{:.2}", value))
                    .footer(serenity::CreateEmbedFooter::new(format!("Этот дейлик будет проходить {} дней. Не забудь забрать завтра +{:.2}, а то пизда", days, value * 1.2)))
                    .color(serenity::Color::ORANGE)
                    .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                    .thumbnail(
                        "https://cdn.discordapp.com/attachments/1185987026365980712/1187009276179386368/stickftoi9reer.webp?ex=65955389&is=6582de89&hm=e4095b724d56c624df92632af2ef31831fa33a39dabd950821e0a288bceb5b1a&"
                    )
                ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
            ).await?;
        }
    }
    Ok(())
}

#[tracing::instrument(name = "generate_new_timely")]
pub async fn generate_new_timely() -> (f64, i8, DateTime<Utc>) {
    let mut rng = rand::thread_rng();
    let value_range = 100.0..150.0;
    let first_value = rng.gen_range(value_range.clone());

    let min_days = 5;

    let days_limit = 30;
    let max_days = ((days_limit as f64 - min_days as f64) / (value_range.end - value_range.start)
        * (value_range.end - first_value)
        + min_days as f64)
        .round() as i8;

    let days = rng.gen_range(min_days..=max_days);
    let timely_end = Utc::now() + Duration::days(days as i64);

    (first_value, days, timely_end)
}
