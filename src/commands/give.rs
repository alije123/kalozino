use ormlite::Model;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::{create_player, database::get_connection, models::database::Player, Context, Error};

///Кинуть деньги в ебало
#[tracing::instrument]
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn give(
    ctx: Context<'_>,
    user_to_give: serenity::User,
    #[min = 0.0000000001]
    #[description = "Сколько денег хочешь кинуть"]
    amount: f64,
) -> Result<(), Error> {
    if amount < 0.0000000001 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ну ты и еблан!")
                    .description("Если ты кинешь отрицательную сумму, то получается чел тебе ещё должен останется, для этого есть steal")
                    .color(serenity::Color::RED),
                ),
            )
            .await?;
        return Ok(());
    }
    let user = ctx.author();
    if user.id == user_to_give.id {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("ваопдлвпоуклдпоплпоук")
                    .description("Нельзя кинуть деньги самому себе")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    }
    let mut db = get_connection().await;
    let user_id = user.id.get() as i64;
    let Some(mut player) = Player::select()
        .where_bind("id = ?", user_id)
        .fetch_optional(&mut db)
        .await?
    else {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Эммм")
                    .description("Как ты можешь кинуть кому-то в ебало денег, если ты даже не участвуешь в каловом казино?")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    };
    if amount > player.balance {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::default()
                    .title("Ну ты вообще придурок")
                    .description("Ты не можешь кинуть больше денег, чем у тебя есть, это создаст чёрную дыру во вселенной")
                    .color(serenity::Color::RED),
            ),
        )
        .await?;
        return Ok(());
    }
    let mut give_player = match Player::select()
        .where_bind("id = ?", user_to_give.id.get() as i64)
        .fetch_optional(&mut db)
        .await?
    {
        Some(x) => x,
        None => create_player(ctx, user.clone(), &mut db).await?,
    };
    player.balance -= amount;
    let new_player_balance = player.balance;
    player.update_all_fields(&mut db).await?;
    give_player.balance += amount;
    let new_give_player_balance = give_player.balance;
    give_player.update_all_fields(&mut db).await?;
    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title(format!("Ты кинул в ебало денег {}", user_to_give.name))
                .description(format!(
                    "Он получил +{:.2}, теперь у него на балансе {:.2}",
                    amount, new_give_player_balance
                ))
                .footer(serenity::CreateEmbedFooter::new(format!(
                    "А у тебя -{:.2}, теперь на балансе: {:.2}",
                    amount, new_player_balance
                )))
                .author(serenity::CreateEmbedAuthor::new(user.name.clone()).icon_url(user.face()))
                .color(serenity::Color::DARK_GREEN),
        ),
    )
    .await?;
    Ok(())
}
