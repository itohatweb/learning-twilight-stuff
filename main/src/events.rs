use std::error::Error;

use simple_process_stats::ProcessStats;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_gateway::Event;
use twilight_model::{application::interaction::Interaction, id::UserId};

use crate::{commands, types::Context};

pub async fn handle(
    context: Context,
    event: Event,
    _shard_id: u64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // if let Err(err) = match event {
    //     Event::InteractionCreate(interaction) => commands::handle(ctx, interaction.0).await,
    //     _ => Err(anyhow!("unknown event: {:?}", event)),
    // } {
    //     eprintln!("{}", err);
    // }
    match event {
        Event::MessageCreate(msg) if msg.content == "++ping" => {
            if !msg
                .author
                .id
                .eq(&UserId::new(615542460151496705_u64).unwrap())
                && !msg
                    .author
                    .id
                    .eq(&UserId::new(275258547749650433_u64).unwrap())
            {
                return Ok(());
            }

            println!("GETTING PROCESS INFO");
            let guild_count = context.cache.iter().guilds().count();
            let channel_count = context.cache.iter().guild_channels().count()
                + context.cache.iter().private_channels().count();
            let message_count = context.cache.iter().messages().count();
            let member_count = context.cache.iter().members().count();
            let role_count = context.cache.iter().roles().count();
            let voice_states_count = context.cache.iter().voice_states().count();

            // use sysinfo::{System, SystemExt};
            // let s = System::new_all();
            // println!("{} KB", s.used_memory());
            let process_stats = ProcessStats::get()
                .await
                .expect("could not get stats for running process");
            println!("flop {:#?}", process_stats);

            // let client = redis::Client::open("redis://127.0.0.1/").unwrap();
            // let mut con = client.get_async_connection().await?;

            // let redis_guilds: u64 = con.hlen("guilds").await?;
            // let redis_channels: u64 = con.hlen("channels").await?;

            // let redis_channels = redis_cache.channels_guild.size().await.unwrap();
            // let redis_guilds = redis_cache.guilds.size().await.unwrap();
            // let redis_messages = redis_cache.messages.size().await.unwrap();
            // let redis_members = redis_cache.members.size().await.unwrap();
            // let redis_roles = redis_cache.roles.size().await.unwrap();

            let embed = EmbedBuilder::new()
                .description("Current statistics of the bot:")
                .field(
                    EmbedFieldBuilder::new(
                        "Cached Stuff:",
                        format!(
                            "guilds: {}\nchannels: {}\nmessages: {}\nmembers: {}\nroles: {}\nvoice states: {}",
                            guild_count,
                            channel_count,
                            message_count,
                            member_count,
                            role_count,
                            voice_states_count
                        ),
                    )
                    .inline(),
                )
                // .field(
                //     EmbedFieldBuilder::new(
                //         "Redis cache:",
                //         format!(
                //             "guilds: {}\nchannels: {}\nmessages: {}\nmembers: {}\nroles: {}",
                //             redis_guilds, redis_channels, redis_messages, redis_members, redis_roles
                //         ),
                //     )
                //     .inline(),
                // )
                .field(EmbedFieldBuilder::new("\u{200B}", "\u{200B}").inline())
                .field(
                    EmbedFieldBuilder::new(
                        "Memory usage:",
                        format!(
                            "{} MB",
                            process_stats.memory_usage_bytes as f64 / 1_000_000.0
                        ),
                    )
                    .inline(),
                )
                .build()?;

            context
                .http
                .create_message(msg.channel_id)
                .embeds(&[embed])?
                .exec()
                .await?;
            // println!(
            //     "---- Message event: {:?}",
            //     serde_json::to_string(&cache.guild(msg.guild_id.unwrap()).as_deref())
            // );

            // let mut missing = vec![];

            // for channel in cache.iter().guild_channels() {
            //     let res = redis_cache
            //         .channels_guild
            //         .includes(channel.id().get())
            //         .await
            //         .unwrap();

            //     if !res {
            //         missing.push((channel.id().get(), format!("{}", channel.name())))
            //     }
            // }

            // http.create_message(msg.channel_id)
            //     .content(&format!("{:#?}", missing))?
            //     .exec()
            //     .await?;
        }
        Event::ShardConnected(connected) => {
            println!("Connected on shard {}", connected.shard_id);
        }
        Event::VoiceStateUpdate(_vsu) => {
            // println!("vsu: {:?}", vsu);
        }
        Event::InteractionCreate(interaction) => {
            context.inter.add_interaction(interaction.id());

            if let Interaction::ApplicationCommand(command) = interaction.0 {
                commands::exec(context, command.as_ref()).await?;
            }
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
