mod commands;
mod events;
mod types;

use dotenv::dotenv;
use futures::stream::StreamExt;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::ResourceType;
use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_model::{gateway::Intents, id::ApplicationId};

use crate::types::InnerContext;

// TODO: look at this cool thing when its finished https://github.com/baptiste0928/twilight-interactions

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let requested_intents = Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_BANS
        | Intents::GUILD_EMOJIS
        | Intents::GUILD_VOICE_STATES
        | Intents::GUILD_INVITES
        | Intents::GUILD_MESSAGE_REACTIONS
        | Intents::DIRECT_MESSAGE_REACTIONS;

    // Load the .env file and just ignore any errors
    dotenv().ok();
    env_logger::init();

    info!("Starting up");

    // debug!("Connecting to Redis server");
    // let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    // let mut con = client.get_async_connection().await?;
    // debug!("Flushing Redis cache");
    // redis::cmd("FLUSHALL")
    //     .query_async::<Connection, ()>(&mut con)
    //     .await?;

    let token = env::var("DISCORD_TOKEN")?;
    let application_id = env::var("BOT_APP_ID")?
        .parse::<u64>()
        .expect("Application Id could not be parsed");
    let application_id = ApplicationId::new(application_id).expect("msg");

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(token.to_owned(), requested_intents)
        .shard_scheme(scheme)
        .build()
        .await?;
    let cluster = Arc::new(cluster);

    // Start up the cluster.
    let cluster_spawn = Arc::clone(&cluster);

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // // HTTP is separate from the gateway, so create a new client.
    // let http = HttpClient::new(token);
    // http.set_application_id(application_id);

    // // Since we only care about new messages, make the cache only
    // // cache new messages.
    // let cache = Arc::new(
    //     InMemoryCache::builder()
    //         .resource_types(ResourceType::all())
    //         .build(),
    // );

    // let redis_cache = Arc::new(InRedisCache::new());

    let context = Arc::new(InnerContext::new(
        token,
        application_id,
        ResourceType::all(),
        cluster.clone(),
    ));

    if false {
        commands::set_dev_commands(&context.http).await?;
    }

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        context.cache.update(&event);
        // redis_cache.update(&event).await;

        tokio::spawn(events::handle(context.clone(), event, shard_id));
    }

    Ok(())
}

// async fn handle_event(
//     shard_id: u64,
//     event: Event,
//     http: TwHttpClient,
//     cache: Arc<InMemoryCache>,
//     redis_cache: Arc<InRedisCache>,
// ) -> Result<(), Box<dyn Error + Send + Sync>> {
//     match event {
//         Event::MessageCreate(msg) if msg.content == "++ping" => {
//             if !msg
//                 .author
//                 .id
//                 .eq(&UserId::new(615542460151496705_u64).unwrap())
//                 && !msg
//                     .author
//                     .id
//                     .eq(&UserId::new(275258547749650433_u64).unwrap())
//             {
//                 return Ok(());
//             }

//             println!("GETTING PROCESS INFO");
//             let guild_count = cache.iter().guilds().count();
//             let channel_count =
//                 cache.iter().guild_channels().count() + cache.iter().private_channels().count();
//             let message_count = cache.iter().messages().count();
//             let member_count = cache.iter().members().count();
//             let role_count = cache.iter().roles().count();
//             let voice_states_count = cache.iter().voice_states().count();

//             // use sysinfo::{System, SystemExt};
//             // let s = System::new_all();
//             // println!("{} KB", s.used_memory());
//             let process_stats = ProcessStats::get()
//                 .await
//                 .expect("could not get stats for running process");
//             println!("flop {:#?}", process_stats);

//             // let client = redis::Client::open("redis://127.0.0.1/").unwrap();
//             // let mut con = client.get_async_connection().await?;

//             // let redis_guilds: u64 = con.hlen("guilds").await?;
//             // let redis_channels: u64 = con.hlen("channels").await?;

//             let redis_channels = redis_cache.channels_guild.size().await.unwrap();
//             let redis_guilds = redis_cache.guilds.size().await.unwrap();
//             let redis_messages = redis_cache.messages.size().await.unwrap();
//             let redis_members = redis_cache.members.size().await.unwrap();
//             let redis_roles = redis_cache.roles.size().await.unwrap();

//             let embed = EmbedBuilder::new()
//                 .description("Current statistics of the bot:")
//                 .field(
//                     EmbedFieldBuilder::new(
//                         "Cached Stuff:",
//                         format!(
//                             "guilds: {}\nchannels: {}\nmessages: {}\nmembers: {}\nroles: {}\nvoice states: {}",
//                             guild_count,
//                             channel_count,
//                             message_count,
//                             member_count,
//                             role_count,
//                             voice_states_count
//                         ),
//                     )
//                     .inline(),
//                 )
//                 .field(
//                     EmbedFieldBuilder::new(
//                         "Redis cache:",
//                         format!(
//                             "guilds: {}\nchannels: {}\nmessages: {}\nmembers: {}\nroles: {}",
//                             redis_guilds, redis_channels, redis_messages, redis_members, redis_roles
//                         ),
//                     )
//                     .inline(),
//                 )
//                 .field(EmbedFieldBuilder::new("\u{200B}", "\u{200B}").inline())
//                 .field(
//                     EmbedFieldBuilder::new(
//                         "Memory usage:",
//                         format!(
//                             "{} MB",
//                             process_stats.memory_usage_bytes as f64 / 1_000_000.0
//                         ),
//                     )
//                     .inline(),
//                 )
//                 .build()?;

//             http.create_message(msg.channel_id)
//                 .embeds(&[embed])?
//                 .exec()
//                 .await?;
//             // println!(
//             //     "---- Message event: {:?}",
//             //     serde_json::to_string(&cache.guild(msg.guild_id.unwrap()).as_deref())
//             // );

//             // let mut missing = vec![];

//             // for channel in cache.iter().guild_channels() {
//             //     let res = redis_cache
//             //         .channels_guild
//             //         .includes(channel.id().get())
//             //         .await
//             //         .unwrap();

//             //     if !res {
//             //         missing.push((channel.id().get(), format!("{}", channel.name())))
//             //     }
//             // }

//             // http.create_message(msg.channel_id)
//             //     .content(&format!("{:#?}", missing))?
//             //     .exec()
//             //     .await?;
//         }
//         Event::ShardConnected(_) => {
//             println!("Connected on shard {}", shard_id);
//         }
//         Event::VoiceStateUpdate(_vsu) => {
//             // println!("vsu: {:?}", vsu);
//         }
//         Event::InteractionCreate(interaction) => {
//             // if let Interaction::ApplicationCommand(command) = interaction.0 {
//             //     exec_command(http, &command, cache).await?;
//             // }
//         }
//         // Other events here...
//         _ => {}
//     }

//     Ok(())
// }
