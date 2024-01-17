use chrono::{Duration, Utc};
use ormlite::Model;
use poise::{
    serenity_prelude::{self as serenity, User},
    CreateReply,
};
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};

use crate::{create_player, database::get_connection, models::database::Player, Context, Error};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn steal(ctx: Context<'_>, user_to_steal: User) -> Result<(), Error> {
    let user = ctx.author();
    if user_to_steal.id == ctx.framework().bot_id {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ты что ахуел??")
                    .description("Какого хуя ты захотел у меня спиздить деньги")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    } else if user_to_steal.bot {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ты чо еблан какой стил ботов")
                    .description("Боты не играют в каволое казино")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    } else if user.id == user_to_steal.id {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Стой стой!")
                    .description("Я уберегаю тебя от впустую потраченной попытки стила самого себя, используй её разумно пж")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    }

    let mut db = get_connection().await;

    let mut player = match Player::select()
        .where_bind("id = ?", ctx.author().id.get() as i64)
        .fetch_optional(&mut db)
        .await?
    {
        Some(x) => x,
        None => create_player(ctx, user.clone(), &mut db).await?,
    };

    if player
        .last_steal_at
        .is_some_and(|x| x + Duration::days(1) > Utc::now())
    {
        let duration_to_next_day =
            (player.last_steal_at.unwrap() + Duration::days(1) - Utc::now()) as Duration;
        let duration_str = format!(
            "{} ч. {} мин.",
            duration_to_next_day.num_hours(),
            (duration_to_next_day - Duration::hours(duration_to_next_day.num_hours()))
                .num_minutes()
        );
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Кажется с последнего стила день ещё не прошёл")
                    .description("Но ты скоро сможешь снова спиздить деньги")
                    .footer(serenity::CreateEmbedFooter::new(format!(
                        "Подожди ещё {}",
                        duration_str
                    )))
                    .color(serenity::Color::RED)
                    .thumbnail("https://cdn.discordapp.com/emojis/769587992972230668.webp?quality=lossless"),
            ),
        )
        .await?;
        return Ok(());
    }

    let mut player_to_steal = match Player::select()
        .where_bind("id = ?", user_to_steal.id.get() as i64)
        .fetch_optional(&mut db)
        .await?
    {
        Some(x) => x,
        None => create_player(ctx, user_to_steal.clone(), &mut db).await?,
    };

    let choices = [StealChoice::Steal, StealChoice::Fail, StealChoice::StealAll];
    let weights = [1.0, 1.0, 0.000001];
    let dist = WeightedIndex::new(&weights)?;

    let steal_choice = {
        let mut rng = rand::thread_rng();
        &choices[dist.sample(&mut rng)]
    };

    match steal_choice {
        StealChoice::Steal => {
            let amount = {
                let mut rng = rand::thread_rng();
                rng.gen_range(60.0..=85.0)
            };

            if player_to_steal.balance < amount {
                ctx.send(
                    CreateReply::default().embed(
                        serenity::CreateEmbed::default()
                            .title("Бедный чел")
                            .description("У него нет денег, а собрался у него пиздить")
                            .footer(serenity::CreateEmbedFooter::new(
                                "Попробуй спиздить у кого-то другого",
                            ))
                            .color(serenity::Color::RED)
                            .author(
                                serenity::CreateEmbedAuthor::new(user_to_steal.name.clone())
                                    .icon_url(user_to_steal.face()),
                            ),
                    ),
                )
                .await?;
                return Ok(());
            }

            player.balance += amount;
            player_to_steal.balance -= amount;

            player.last_steal_at = Some(Utc::now());

            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title(format!(
                            "Ты успешно спиздил {:.2} деньжат у {}",
                            amount, user_to_steal.name
                        ))
                        .description(format!(
                            "Теперь у тебя на балансе {:.2}, а у {} - {:.2}",
                            player.balance, user_to_steal.name, player_to_steal.balance
                        ))
                        .footer(serenity::CreateEmbedFooter::new(
                            "Следующая попытка будет через день",
                        ))
                        .color(serenity::Color::DARK_GREEN)
                        .author(
                            serenity::CreateEmbedAuthor::new(user.name.clone())
                                .icon_url(user.face()),
                        ),
                ),
            )
            .await?;
        }
        StealChoice::Fail => {
            player.last_steal_at = Some(Utc::now());
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title("Сегодня не получилось спиздить")
                        .description("Попробуй завтра")
                        .footer(serenity::CreateEmbedFooter::new(
                            "Следующая попытка будет через день",
                        ))
                        .color(serenity::Color::RED),
                ),
            )
            .await?;
        }
        StealChoice::StealAll => {
            player.balance += player_to_steal.balance;
            player.last_steal_at = Some(Utc::now());
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("АХАХАХАХАХАХАХАХА")
                    .description(format!(
                        "Ты спиздил у {} все ({:.2})  деньги",
                        user_to_steal.name, player_to_steal.balance
                    ))
                    .footer(serenity::CreateEmbedFooter::new(format!("Теперь у тебя на балансе {:.2}, а у {} нет денег вообще. Следующая попытка будет через день",
                        player.balance, user_to_steal.name)))
                    .color(serenity::Color::DARK_GREEN)
                    .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face()))),)
            .await?;
        }
    }

    player.update_all_fields(&mut db).await?;
    player_to_steal.update_all_fields(&mut db).await?;

    Ok(())
}

pub enum StealChoice {
    Steal,
    Fail,
    StealAll,
}
