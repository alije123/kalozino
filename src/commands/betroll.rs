use ormlite::{
    model::{HasModelBuilder, ModelBuilder},
    Model,
    postgres::PgConnection,
};
use poise::serenity_prelude as serenity;
use rand::Rng;

use crate::{
    Context,
    database::get_connection,
    Error,
    models::database::Player, utils::{create_player, send_default_message_with_face},
};

/// Проебать свои деньги, или же выиграть больше?
#[tracing::instrument]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    aliases("br"),
    help_text_fn = "help_text"
)]
pub async fn betroll(
    ctx: Context<'_>,
    #[description = "Сколько денег хочешь проебать"]
    #[min = 0.0000000001]
    bet: f64,
) -> Result<(), Error> {
    let mut embed = serenity::CreateEmbed::default();
    let user = ctx.author();

    if bet == 0.0 {
        embed = embed
            .title("Ошибка")
            .description("Нельзя проебать 0 денег")
            .color(serenity::Color::RED);
    } else if bet < 0.0 {
        embed = embed
            .title("Ошибка")
            .description("Нельзя ставить отрицательно")
            .color(serenity::Color::RED);
    } else {
        let mut db = get_connection().await;
        let user = ctx.author();
        let user_id = user.id.get() as i64;

        let player = match Player::select()
            .where_bind("id = ?", user_id)
            .fetch_optional(&mut db)
            .await?
        {
            Some(x) => x,
            None => create_player(ctx, user.clone(), &mut db).await?,
        };

        if !(bet > player.balance) {
            embed = process_betroll(bet, player, &mut db).await?;
        } else {
            embed = embed
                .title("Ошибка")
                .description("Нельзя проебать больше, чем у тебя есть")
                .color(serenity::Color::RED);
        }
    }

    send_default_message_with_face(ctx, embed, user).await?;

    Ok(())
}

async fn process_betroll(
    bet: f64,
    player: Player,
    db: &mut PgConnection,
) -> Result<serenity::builder::CreateEmbed, Error> {
    let mut embed = serenity::CreateEmbed::default();

    let roll = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..=101)
    };

    let reward_multiplier = match roll {
        0 => PossibleRewards::Back,
        66..=89 => PossibleRewards::Two,
        90..=92 => PossibleRewards::Four,
        93..=98 => PossibleRewards::Six,
        99.. => PossibleRewards::Ten,
        _ => PossibleRewards::None,
    };

    let mut balance = player.balance;

    match reward_multiplier {
        PossibleRewards::None => {
            let multiplier = -1.0;
            let delta = bet * multiplier;
            balance += delta;
            let pic_url = "https://cdn.discordapp.com/attachments\
            /1185987026365980712/1187416807510921389\
            /dd6da43f4c650568.png?ex=6596cf14&is=65845a14\
            &hm=97ace7cd7675207bf7d172834542a9b8d15a4d5d1049c886f82a028801360c3a&";
            embed = embed
                .title("中国党很生气，你做了坏事就忏悔。")
                .description("你的钱留在国库里")
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "你搞砸了 {:.2} 你现在的账户是 {:.2}",
                    delta, balance
                )))
                .color(serenity::Color::RED)
                .thumbnail(pic_url);
        }
        PossibleRewards::Back => {
            let multiplier = 0.0;
            let delta = bet * multiplier;
            balance += delta;
            let pic_url = "https://cdn.discordapp.com/attachments\
            /1185987026365980712/1187416807917748264\
            /ccf4d09fd133a480.png?ex=6596cf14&is=65845a14\
            &hm=a2e744b1a1591d2572e2dd2ef8a41f33c8533db01aa06268da21cc4423077618&";
            embed = embed
                .title("1х. Ты ничо не выиграл но и не проебал")
                .description(format!("Твоя деньги ({:.2}) остаются у тебя", bet))
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Баланс не изменился: {:.2}",
                    balance
                )))
                .color(serenity::Color::ROSEWATER)
                .thumbnail(pic_url);
        }
        PossibleRewards::Two => {
            let multiplier = 1.0;
            let delta = bet * multiplier;
            balance += delta;
            let pic_url = "https://cdn.discordapp.com/attachments\
            /1185987026365980712/1187416805736730705\
            /2.png?ex=6596cf13&is=65845a13\
            &hm=7f6b359e4b1350d1f32a258a861f6ee72d5bb8070df4f4a94bbf3da50f62741d&";
            embed = embed
                .title("Китай партия выдать тебе 2х")
                .description("К твоему балансу прибавить сумма ставки")
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Ты выиграть +{:.2}. На твоем счету теперь: {:.2}",
                    delta, balance
                )))
                .color(serenity::Color::DARK_GREEN)
                .thumbnail(pic_url);
        }
        PossibleRewards::Four => {
            let multiplier = 3.0;
            let delta = bet * multiplier;
            balance += delta;
            let pic_url = "https://cdn.discordapp.com/attachments\
            /1185987026365980712/1187416807158579321/4.png\
            ?ex=6596cf13&is=65845a13\
            &hm=ffdaf63ba4985b5c5d6207829476ad5d8922f80d0ab7d22ded1ee8a1a21dc146&";
            embed = embed
                .title("Китай партия выдать тебе 4х")
                .description("К твоему балансу прибавить 3х сумму ставки")
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Ты выиграть +{:.2}. На твоем счету теперь: {:.2}",
                    delta, balance
                )))
                .color(serenity::Color::DARK_GREEN)
                .thumbnail(pic_url);
        }
        PossibleRewards::Six => {
            let multiplier = 5.0;
            let delta = bet * multiplier;
            balance += delta;
            let pic_url = "https://cdn.discordapp.com/attachments\
            /1185987026365980712/1187416806810472614/4-1.png\
            ?ex=6596cf13&is=65845a13\
            &hm=591fbb82c0fdd9b1956edbbfdf6739e65bfa8f67c1e37a40ac0e0ff5b3c03f7b&";
            embed = embed
                .title("ОГО! 6х!")
                .description("Китай партия к твоему балансу прибавить 6х сумму ставки")
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Ты выиграть +{:.2}. На твоем счету теперь: {:.2}",
                    delta, balance
                )))
                .color(serenity::Color::PURPLE)
                .thumbnail(pic_url);
        }
        PossibleRewards::Ten => {
            let multiplier = 9.0;
            let delta = bet * multiplier;
            balance += delta;
            let pic_url = "https://cdn.discordapp.com/attachments\
            /1185987026365980712/1187416806147760258/10.png\
            ?ex=6596cf13&is=65845a13\
            &hm=51f5fe059a59f281ccf0db8e7cc7a1da87036642cc9dd65752d76c2beb808fc5&";
            embed = embed
                .title("МЕГА ВИН! 10х! АХУЕТЬ!")
                .description("操Китай партия к твоему балансу прибавить 10х сумму ставки")
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "Ты выиграть +{:.2}. На твоем счету теперь: {:.2}",
                    delta, balance
                )))
                .color(serenity::Color::GOLD)
                .thumbnail(pic_url);
        }
    }

    player.update_partial().balance(balance).update(db).await?;

    Ok(embed)
}

pub enum PossibleRewards {
    None = 0,
    Back = 1,
    Two = 2,
    Four = 4,
    Six = 6,
    Ten = 10,
}

fn help_text() -> String {
    serenity::MessageBuilder::new()
        .push("Ты вообще не ебёшь как команды писать? Бери и пиши: ")
        .push_mono("(/|!)betroll|br сумма-хуюмма")
        .build()
}
