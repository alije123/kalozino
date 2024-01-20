#![feature(async_closure)]

use std::sync::Arc;

use crate::models::database::{ActiveCustomVoice, Player};
use database::get_connection;
use models::{config::CustomVoice, database::Config};
use ormlite::{postgres::PgConnection, Connection, Model};
use poise::{serenity_prelude as serenity, CreateReply, FrameworkError};
use tokio::sync::RwLock;
use tracing_subscriber::{
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub mod commands;
pub mod database;
pub mod models;

#[derive(Debug, Clone)]
pub struct Data {}

// User data, which is stored and accessible in all command invocations
type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn create_player(
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

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let appender_layer = fmt::layer()
        .compact()
        // disable any ansi escape sequences, otherwise will be written to file
        .with_ansi(false)
        .with_timer(ChronoLocal::rfc_3339())
        .with_writer(tracing_appender::rolling::daily("./logs", "bot-log"));

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_timer(ChronoLocal::rfc_3339()).pretty())
        .with(appender_layer)
        .init();
    PgConnection::connect(&std::env::var("DATABASE_URL").expect("missing DATABASE_URL"))
        .await
        .expect("failed to connect to database");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::balance::balance(),
                commands::timely::timely(),
                commands::bankroll::bankroll(),
                commands::reset::reset(),
                commands::steal::steal(),
                commands::set_custom_voice_spawn::set_custom_voice_spawn(),
                commands::top::top(),
                commands::give::give(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |error| Box::pin(on_error(error)),
            pre_command: |ctx| {
                Box::pin(async move {
                    let _ = ctx.defer_or_broadcast().await;
                    tracing::info!(
                        "{}({}) executed command {}",
                        ctx.author().name,
                        ctx.author().id,
                        ctx.invoked_command_name()
                    );
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                tracing::info!("Ready! @{}", _ready.user.name);
                serenity::ChannelId::new(812393660485992461)
                    .say(&ctx, "рет пидор")
                    .await?;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::privileged()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    if let Err(e) = match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(
                "Error in command `{}:` {:?}",
                ctx.command().qualified_name,
                error
            );
            ctx.send(
                CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title("Ой! Ошибка!")
                        .description(&error.to_string())
                        .color(serenity::Color::RED),
                ),
            )
            .await
            .map(|_| ())
        }
        crate::FrameworkError::EventHandler { error, event, .. } => {
            tracing::error!(
                "User event event handler encountered an error on {:?} event: {:?}",
                event,
                error
            );
            Ok(())
        }
        error => poise::builtins::on_error(error).await,
    } {
        tracing::error!("Error while handling error: {:?}", e);
    }
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::VoiceStateUpdate { new, .. } => {
            let Some(guild_id) = new.guild_id else {
                return Ok(());
            };
            let guild = guild_id.to_partial_guild(&ctx).await?;

            let user = new.user_id.to_user(&ctx).await?;
            let member = guild.member(&ctx, new.user_id).await?;

            let mut db = get_connection().await;
            let custom_voice_config = Config::select()
                .where_bind("key = ?", "custom_voice")
                .fetch_one(&mut db)
                .await?;
            let custom_voice_config: CustomVoice =
                serde_json::from_value(custom_voice_config.data.0)?;
            let Some(spawn_channel) =
                serenity::ChannelId::new(custom_voice_config.voice_channel_id)
                    .to_channel(&ctx)
                    .await?
                    .guild()
            else {
                return Ok(());
            };

            let custom_voices = ActiveCustomVoice::select().fetch_all(&mut db).await?;
            let Some(channel_id) = new.channel_id else {
                check_custom_voices(
                    custom_voices,
                    // custom_voice_channel_with_delete_error,
                    ctx,
                    &mut db,
                    &spawn_channel,
                    &user,
                )
                .await?;
                return Ok(());
            };
            if custom_voice_config.voice_channel_id != channel_id.get() {
                check_custom_voices(
                    custom_voices,
                    // custom_voice_channel_with_delete_error,
                    ctx,
                    &mut db,
                    &spawn_channel,
                    &user,
                )
                .await?;

                return Ok(());
            }
            tracing::info!("{} joined spawn voice channel", user.name);

            let mut player = match Player::select()
                .where_bind("id = ?", user.id.get() as i64)
                .fetch_optional(&mut db)
                .await?
            {
                Some(x) => x,
                None => {
                    tracing::info!("{} not found as player, kicking!", user.name);
                    member.disconnect_from_voice(&ctx).await?;
                    return Ok(());
                }
            };
            let Some(category) = serenity::ChannelId::from(custom_voice_config.category_id)
                .to_channel(&ctx)
                .await?
                .category()
            else {
                tracing::info!("I can't retrieve category of spawn voice, kicking!");
                member.disconnect_from_voice(&ctx).await?;
                return Ok(());
            };
            if player.balance < 1.0 {
                tracing::info!("{} not enough balance, kicking!", user.name);
                member.disconnect_from_voice(&ctx).await?;
                return Ok(());
            }
            let new_voice = guild
                .create_channel(
                    &ctx,
                    serenity::CreateChannel::new(format!(
                        "Войс имени {}",
                        user.global_name.unwrap_or(user.name.clone())
                    ))
                    .permissions([serenity::PermissionOverwrite {
                        allow: serenity::Permissions::MANAGE_CHANNELS
                            | serenity::Permissions::MOVE_MEMBERS
                            | serenity::Permissions::CONNECT
                            | serenity::Permissions::SPEAK
                            | serenity::Permissions::VIEW_CHANNEL,
                        deny: serenity::Permissions::empty(),
                        kind: serenity::PermissionOverwriteType::Member(user.id),
                    }])
                    .category(category.id)
                    .kind(serenity::ChannelType::Voice),
                )
                .await?;

            ActiveCustomVoice {
                id: new_voice.id.get() as i64,
                owner_id: user.id.get() as i64,
            }
            .insert(&mut db)
            .await?;
            player.balance -= 1.0;
            player.update_all_fields(&mut db).await?;

            tracing::info!("Moving {} to new voice", user.name);
            member.move_to_voice_channel(&ctx, new_voice).await?;
            spawn_channel
                .create_permission(
                    &ctx,
                    serenity::PermissionOverwrite {
                        allow: serenity::Permissions::empty(),
                        deny: serenity::Permissions::CONNECT | serenity::Permissions::VIEW_CHANNEL,
                        kind: serenity::PermissionOverwriteType::Member(user.id),
                    },
                )
                .await?;
        }
        _ => {}
    }
    Ok(())
}

async fn check_custom_voices(
    custom_voices: Vec<ActiveCustomVoice>,
    ctx: &serenity::Context,
    db: &mut PgConnection,
    spawn_channel: &serenity::GuildChannel,
    user: &serenity::User,
) -> Result<(), anyhow::Error> {
    for custom_voice in custom_voices {
        let Some(guild_channel) = serenity::ChannelId::new(custom_voice.id as u64)
            .to_channel(&ctx)
            .await?
            .guild()
        else {
            continue;
        };
        let members = guild_channel.members(&ctx)?;
        if members.is_empty() {
            tracing::info!(
                "{}({}) is empty, deleting",
                guild_channel.name,
                guild_channel.id
            );
            let delete_result = guild_channel.delete(&ctx).await;
            match delete_result {
                Err(e) => {
                    tracing::error!("Failed to delete voice channel: {:?}", e);
                    continue;
                }
                Ok(_) => {
                    custom_voice.delete(&mut *db).await?;
                }
            }
            let _ = spawn_channel
                .delete_permission(&ctx, serenity::PermissionOverwriteType::Member(user.id))
                .await;
        }
    }
    Ok(())
}
