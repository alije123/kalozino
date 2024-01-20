use ormlite::Model;
use poise::{
    serenity_prelude::{self as serenity},
    CreateReply,
};
use rand::distributions::{Distribution, WeightedIndex};

use crate::{create_player, database::get_connection, models::database::Player, Context, Error};

/// Проебать свои деньги, или же выиграть больше?
#[tracing::instrument]
#[poise::command(slash_command, prefix_command, guild_only, aliases("br"))]
pub async fn bankroll(
    ctx: Context<'_>,
    #[description = "Сколько денег хочешь проебать"]
    #[min = 0.0000000001]
    bet: f64,
) -> Result<(), Error> {
    if bet == 0.0 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ошибка")
                    .description("Нельзя проебать 0 денег")
                    .color(serenity::Color::RED),
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
        .await?;
        return Ok(());
    }

    if bet < 0.0 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ошибка")
                    .description("Нельзя ставить отрицательно")
                    .color(serenity::Color::RED),
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
        .await?;
        return Ok(());
    }

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

    if bet > player.balance {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ошибка")
                    .description("Нельзя проебать больше, чем у тебя есть")
                    .color(serenity::Color::RED)
                    .author(
                        serenity::CreateEmbedAuthor::new(user.name.clone()).icon_url(user.face()),
                    ),
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
        )
        .await?;
        return Ok(());
    }

    let choices = [
        PossibleRewards::None,
        PossibleRewards::Back,
        PossibleRewards::Two,
        PossibleRewards::Four,
        PossibleRewards::Six,
        PossibleRewards::Ten,
    ];
    let weights = [8.0, 0.0008, 8.0, 0.1, 0.001, 0.00002];
    let dist = WeightedIndex::new(&weights)?;

    let reward_multiplier = {
        let mut rng = rand::thread_rng();
        &choices[dist.sample(&mut rng)]
    };

    match reward_multiplier {
        PossibleRewards::None => {
            player.balance -= bet;
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("中国党很生气，你做了坏事就忏悔。")
                            .description("你的钱留在国库里")
                            .footer(serenity::CreateEmbedFooter::new(format!("你搞砸了 -{:.2} 你现在的账户是 {:.2}", bet, player.balance)))
                            .color(serenity::Color::RED)
                            .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                            .thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1187416807510921389/dd6da43f4c650568.png?ex=6596cf14&is=65845a14&hm=97ace7cd7675207bf7d172834542a9b8d15a4d5d1049c886f82a028801360c3a&")
                        ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
                    ).await?;
        }
        PossibleRewards::Back => {
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("1х. Ты ничо не выиграл но и не проебал")
                            .description(format!("Твоя деньги ({:.2}) остаются у тебя", bet))
                            .footer(serenity::CreateEmbedFooter::new(format!("Баланс не изменился: {:.2}", player.balance)))
                            .color(serenity::Color::ROSEWATER)
                            .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                            .thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1187416807917748264/ccf4d09fd133a480.png?ex=6596cf14&is=65845a14&hm=a2e744b1a1591d2572e2dd2ef8a41f33c8533db01aa06268da21cc4423077618&")
            ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false))).await?;
        }
        PossibleRewards::Two => {
            player.balance += bet * 2.0;
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("Китай партия выдать тебе 2х")
                    .description("К твоему балансу прибавить сумма ставки")
                    .footer(serenity::CreateEmbedFooter::new(format!("Ты выиграть +{:.2}. На твоем счету теперь: {:.2}", bet * 2.0, player.balance)))
                    .color(serenity::Color::DARK_GREEN)
                    .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                    .thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1187416805736730705/2.png?ex=6596cf13&is=65845a13&hm=7f6b359e4b1350d1f32a258a861f6ee72d5bb8070df4f4a94bbf3da50f62741d&")
                ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
            )
            .await?;
        }
        PossibleRewards::Four => {
            player.balance += bet * 4.0;
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("Китай партия выдать тебе 4х")
                    .description("К твоему балансу прибавить 4х сумму ставки")
                    .footer(serenity::CreateEmbedFooter::new(format!("Ты выиграть +{:.2}. На твоем счету теперь: {:.2}", bet * 4.0, player.balance)))
                    .color(serenity::Color::DARK_GREEN)
                    .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                    .thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1187416807158579321/4.png?ex=6596cf13&is=65845a13&hm=ffdaf63ba4985b5c5d6207829476ad5d8922f80d0ab7d22ded1ee8a1a21dc146&")
                ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
            ).await?;
        }
        PossibleRewards::Six => {
            player.balance += bet * 6.0;
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("ОГО! 6х!")
                    .description("Китай партия к твоему балансу прибавить 6х сумму ставки")
                    .footer(serenity::CreateEmbedFooter::new(format!("Ты выиграть +{:.2}. На твоем счету теперь: {:.2}", bet * 6.0, player.balance)))
                    .color(serenity::Color::PURPLE)
                    .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                    .thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1187416806810472614/4-1.png?ex=6596cf13&is=65845a13&hm=591fbb82c0fdd9b1956edbbfdf6739e65bfa8f67c1e37a40ac0e0ff5b3c03f7b&")
                ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
            )
            .await?;
        }
        PossibleRewards::Ten => {
            player.balance += bet * 10.0;
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default().title("МЕГА ВИН! 10х! АХУЕТЬ!")
                    .description("操Китай партия к твоему балансу прибавить 10х сумму ставки")
                    .footer(serenity::CreateEmbedFooter::new(format!("Ты выиграть +{:.2}. На твоем счету теперь: {:.2}", bet * 10.0, player.balance)))
                    .color(serenity::Color::GOLD)
                    .author(serenity::CreateEmbedAuthor::new(user.name.clone())
                .icon_url(user.face())
            )
                    .thumbnail("https://cdn.discordapp.com/attachments/1185987026365980712/1187416806147760258/10.png?ex=6596cf13&is=65845a13&hm=51f5fe059a59f281ccf0db8e7cc7a1da87036642cc9dd65752d76c2beb808fc5&")
                ).reply(true).allowed_mentions(serenity::CreateAllowedMentions::default().replied_user(false)),
            )
            .await?;
        }
    }

    player.update_all_fields(&mut db).await?;
    Ok(())
}

pub enum PossibleRewards {
    None = 0,
    Back = 1,
    Two = 2,
    Four = 4,
    Six = 6,
    Ten = 10,
}
