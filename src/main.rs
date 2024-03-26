use ormlite::{Connection, Model, postgres::PgConnection};
use poise::{CreateReply, FrameworkError, serenity_prelude as serenity};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use database::get_connection;
use models::{config::CustomVoice, database::Config};

use crate::models::config::Starboard;
use crate::models::database::{ActiveCustomVoice, Player, StarboardMessage};

pub mod commands;
pub mod database;
pub mod models;
mod utils;

#[derive(Debug, Clone)]
pub struct Data {}

// User data, which is stored and accessible in all command invocations
type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, Data, Error>;

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
                commands::betroll::betroll(),
                commands::reset::reset(),
                commands::steal::steal(),
                commands::top::top(),
                commands::give::give(),
                commands::set::set(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                additional_prefixes: vec![poise::Prefix::Literal("/")],
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
                serenity::ChannelId::new(1207564679979999232)
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
        | serenity::GatewayIntents::GUILD_PRESENCES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}

async fn on_error(error: FrameworkError<'_, Data, Error>) {
    if let Err(e) = match error {
        FrameworkError::Setup { error, .. } => {
            tracing::error!("Error in user data setup: {:?}", error);
            Ok(())
        }
        FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(
                "Error in command `{}:` {:?}",
                ctx.command().qualified_name,
                error
            );
            ctx.send(
                CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::default()
                            .title("Ой! Ошибка!")
                            .description(&error.to_string())
                            .color(serenity::Color::RED),
                    )
                    .reply(true)
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::default().replied_user(false),
                    ),
            )
            .await
            .map(|_| ())
        }
        FrameworkError::EventHandler { error, event, .. } => {
            tracing::error!(
                "User event event handler encountered an error on {:?} event: {:?}",
                event,
                error
            );
            Ok(())
        }
        FrameworkError::ArgumentParse {
            ctx, input, error, ..
        } => {
            // If we caught an argument parse error, give a helpful error message with the
            // command explanation if available
            let usage = match &ctx.command().help_text {
                Some(help_text) => &**help_text,
                None => "Губы губа губами",
            };
            let response = if let Some(input) = input {
                format!("**Что это за хуйня: `{}`, {}**\n{}", input, error, usage)
            } else {
                format!("**{}**\n{}", error, usage)
            };
            ctx.say(response).await.map(|_| ())
        }
        error => poise::builtins::on_error(error).await,
    } {
        tracing::error!("Error in framework: {:?}", e);
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
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            let Some(guild_id) = add_reaction.guild_id else {
                return Ok(());
            };

            let mut db = get_connection().await;

            let Some(starboard_config) = Config::select()
                .where_bind("key = 'starboard' and server_id = ?", guild_id.get() as i64)
                .fetch_optional(&mut db)
                .await?
            else {
                tracing::info!("no starboard config found for server {}", guild_id);
                return Ok(());
            };
            let starboard_config: Starboard = serde_json::from_value(starboard_config.data.0)?;

            let channel_id = add_reaction.channel_id;

            if starboard_config
                .channels_whitelist
                .contains(&channel_id.get())
            {
                tracing::info!("channel in whitelist, ignoring");
                return Ok(());
            }
            if starboard_config.forward_channel_id == channel_id.get() {
                tracing::info!("channel equal forward channel, ignoring");
                return Ok(());
            }

            let message = add_reaction.message(ctx).await?;

            let reaction_count = message
                .reaction_users(ctx, add_reaction.emoji.clone(), None, None)
                .await?
                .len();

            if reaction_count < starboard_config.reaction_threshold {
                tracing::info!(
                    "{} < {} reactions, ignoring",
                    reaction_count,
                    starboard_config.reaction_threshold
                );
                return Ok(());
            }

            if let Some(existing_message) = StarboardMessage::select()
                .where_bind("message_id = ?", message.id.get() as i64)
                .fetch_optional(&mut db)
                .await?
            {
                tracing::info!("starboard message already exists");
                let Some(starboard_channel) =
                    serenity::ChannelId::new(starboard_config.forward_channel_id)
                        .to_channel(ctx)
                        .await?
                        .guild()
                else {
                    tracing::error!("failed to get starboard channel");
                    return Ok(());
                };
                let mut existing_message_from_discord = starboard_channel
                    .message(
                        ctx,
                        serenity::MessageId::new(existing_message.message_id as u64),
                    )
                    .await?;

                let last_reaction_count = existing_message.last_reaction_count;
                if reaction_count != last_reaction_count as usize {
                    tracing::info!("updating starboard message reaction counter");
                    let new_text = message.content.replace(
                        &last_reaction_count.to_string(),
                        &reaction_count.to_string(),
                    );

                    existing_message_from_discord
                        .edit(ctx, serenity::EditMessage::default().content(new_text))
                        .await?;
                }
                return Ok(());
            }

            let partial_guild = guild_id.to_partial_guild(ctx).await?;
            tracing::info!("getting reaction guild");

            let emoji = match add_reaction.clone().emoji {
                serenity::ReactionType::Custom { id, name, .. } => {
                    if let Ok(guild_emoji) = partial_guild.emoji(ctx, id).await {
                        EmojiType::Custom(guild_emoji)
                    } else {
                        EmojiType::Text(format!(
                            ":{}:",
                            name.unwrap_or("хз чо за реакция".to_string())
                        ))
                    }
                }
                serenity::ReactionType::Unicode(str) => EmojiType::Text(str),
                _ => EmojiType::Text(":хз чо за реакция:".to_string()),
            };

            let mut message_builder = serenity::MessageBuilder::new();
            match emoji {
                EmojiType::Custom(emoji) => message_builder.emoji(&emoji),
                EmojiType::Text(str) => message_builder.push(str),
            };
            message_builder.push(format!("{} в ", reaction_count.to_string()));
            message_builder.channel(channel_id).build();

            let mut starboard_message =
                serenity::CreateMessage::default().content(message_builder.build());

            starboard_message = starboard_message.button(
                serenity::CreateButton::new_link(message.link().clone()).label("Сообщение"),
            );

            if message.kind == serenity::MessageType::InlineReply {
                let reply_message = message.referenced_message.clone().unwrap();
                let reply_embed =
                    copy_message_to_embed(reply_message.clone(), StarboardEmbedType::Reply);
                starboard_message = starboard_message.add_embed(reply_embed);

                starboard_message = starboard_message.button(
                    serenity::CreateButton::new_link(reply_message.link().clone())
                        .label("Отвеченное сообщение"),
                );
            }

            message
                .attachments
                .clone()
                .iter()
                .filter(|attachment| {
                    attachment
                        .content_type
                        .clone()
                        .and_then(|content_type| {
                            if !content_type.contains("image") {
                                Some(())
                            } else {
                                None
                            }
                        })
                        .is_some()
                })
                .for_each(|attachment| {
                    starboard_message = starboard_message.clone().button(
                        serenity::CreateButton::new_link(attachment.url.clone())
                            .label(attachment.filename.clone()),
                    );
                });

            let embed =
                copy_message_to_embed(Box::from(message.clone()), StarboardEmbedType::Original);
            starboard_message = starboard_message.add_embed(embed);

            let sent_message = serenity::ChannelId::new(starboard_config.forward_channel_id)
                .send_message(ctx, starboard_message)
                .await?;

            StarboardMessage {
                message_id: message.id.get() as i64,
                server_id: guild_id.get() as i64,
                forwarded_message_id: sent_message.id.get() as i64,
                last_reaction_count: reaction_count as i16,
            }
            .insert(&mut db)
            .await?;
        }
        serenity::FullEvent::GuildMemberUpdate { new, event, .. } => {
            if event.guild_id.get() != 1184271056148631574 {
                return Ok(());
            }

            let Some(member) = new else {
                return Ok(());
            };

            if member.user.id.get() != 464355896592695296 {
                return Ok(());
            }

            if !member.roles.is_empty() {
                member.remove_roles(ctx, &member.roles).await?;
            }
        }
        serenity::FullEvent::ChannelUpdate { new, .. } => {
            let mut channel = new.clone();
            if (new.name.contains("алиже") || new.name.contains("alije"))
                && new.name.contains("пидор")
            {
                channel
                    .edit(
                        &ctx,
                        serenity::EditChannel::default().name("голден пидор✅"),
                    )
                    .await?;
            }

            if new.topic.is_some()
                && (new.topic.clone().unwrap().contains("алиже")
                    || new.topic.clone().unwrap().contains("alije"))
                && new.topic.clone().unwrap().contains("пидор")
            {
                channel
                    .edit(
                        &ctx,
                        serenity::EditChannel::default().topic("голден пидор✅"),
                    )
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}

enum StarboardEmbedType {
    Reply,
    Original,
}

fn copy_message_to_embed(
    message: Box<serenity::Message>,
    starboard_embed_type: StarboardEmbedType,
) -> serenity::CreateEmbed {
    let mut reply_embed = serenity::CreateEmbed::default();
    let author_string = match starboard_embed_type {
        StarboardEmbedType::Reply => {
            format!("Пиздит на {}", message.author.name.clone())
        }
        StarboardEmbedType::Original => message.author.name.clone(),
    };
    reply_embed = reply_embed
        .author(
            serenity::CreateEmbedAuthor::new(author_string)
                .icon_url(message.author.face())
                .url(message.link()),
        )
        .description(message.content)
        .timestamp(message.timestamp)
        .color(match starboard_embed_type {
            StarboardEmbedType::Reply => serenity::Color::default(),
            StarboardEmbedType::Original => serenity::Color::ROHRKATZE_BLUE,
        });
    message
        .attachments
        .iter()
        .filter(|attachment| {
            attachment
                .content_type
                .clone()
                .and_then(|content_type| {
                    if content_type.contains("image") {
                        Some(())
                    } else {
                        None
                    }
                })
                .is_some()
        })
        .for_each(|attachment| {
            reply_embed = reply_embed.clone().image(attachment.url.clone());
        });
    message.embeds.iter().for_each(|embed| {
        reply_embed = reply_embed.clone().field(
            embed.title.clone().unwrap_or("".to_string()),
            embed.description.clone().unwrap_or("".to_string()),
            false,
        );
        if let Some(image) = embed.image.clone() {
            reply_embed = reply_embed.clone().image(image.url);
        }
        if let Some(thumbnail) = embed.thumbnail.clone() {
            reply_embed = reply_embed.clone().thumbnail(thumbnail.url);
        }
        embed.fields.iter().for_each(|field| {
            reply_embed = reply_embed
                .clone()
                .field(&field.name, &field.value, field.inline);
        })
    });
    reply_embed
}

enum EmojiType {
    Custom(serenity::Emoji),
    Text(String),
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
