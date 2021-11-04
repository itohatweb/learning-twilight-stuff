use bot_test::{commands::exec_command, types::TwHttpClient};
use cache::{InRedisCache, RedisSetCache};
use dotenv::dotenv;
use futures::stream::StreamExt;
use redis::{aio::Connection, AsyncCommands};
use simple_process_stats::ProcessStats;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::Interaction, channel::GuildChannel, gateway::Intents, id::UserId,
};

// TODO: look at this cool thing when its finished https://github.com/baptiste0928/twilight-interactions

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Load the .env file and just ignore any errors
    dotenv().ok();
    env_logger::init();

    info!("Starting up");

    debug!("Connecting to Redis server");
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await?;
    debug!("Flushing Redis cache");
    redis::cmd("FLUSHALL")
        .query_async::<Connection, ()>(&mut con)
        .await?;

    let token = env::var("DISCORD_TOKEN")?;

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(
        token.to_owned(),
        Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::DIRECT_MESSAGES
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_BANS
            | Intents::GUILD_EMOJIS
            | Intents::GUILD_VOICE_STATES
            | Intents::GUILD_INVITES
            | Intents::GUILD_MESSAGE_REACTIONS
            | Intents::DIRECT_MESSAGE_REACTIONS,
    )
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

    // HTTP is separate from the gateway, so create a new client.
    let http = Arc::new(HttpClient::new(token));

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(ResourceType::all())
            .build(),
    );

    let redis_cache = Arc::new(InRedisCache::new());

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);
        redis_cache.update(&event).await;

        tokio::spawn(handle_event(
            shard_id,
            event,
            Arc::clone(&http),
            cache.clone(),
            redis_cache.clone(),
        ));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: TwHttpClient,
    cache: Arc<InMemoryCache>,
    redis_cache: Arc<InRedisCache>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
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
            let guild_count = cache.iter().guilds().count();
            let channel_count =
                cache.iter().guild_channels().count() + cache.iter().private_channels().count();
            let message_count = cache.iter().messages().count();
            let member_count = cache.iter().members().count();
            let voice_states_count = cache.iter().voice_states().count();

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

            let redis_channels = redis_cache.channels_guild.size().await.unwrap();
            let redis_guilds = redis_cache.guilds.size().await.unwrap();

            let embed = EmbedBuilder::new()
                .description("Current statistics of the bot:")
                .field(
                    EmbedFieldBuilder::new(
                        "Cached Stuff:",
                        format!(
                            "guilds: {}\nchannels: {}\nmessages: {}\nmembers: {}\nvoice states: {}",
                            guild_count,
                            channel_count,
                            message_count,
                            member_count,
                            voice_states_count
                        ),
                    )
                    .inline(),
                )
                .field(
                    EmbedFieldBuilder::new(
                        "Redis cache:",
                        format!("guilds: {}\nchannels: {}", redis_guilds, redis_channels),
                    )
                    .inline(),
                )
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

            http.create_message(msg.channel_id)
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
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        Event::VoiceStateUpdate(_vsu) => {
            // println!("vsu: {:?}", vsu);
        }
        Event::InteractionCreate(interaction) => {
            if let Interaction::ApplicationCommand(command) = interaction.0 {
                exec_command(http, &command, cache).await?;
            }
        }
        Event::GuildCreate(guild) => {
            // let client = redis::Client::open("redis://127.0.0.1/").unwrap();
            // let mut con = client.get_async_connection().await?;

            // let bytes = bincode::serialize(&guild).unwrap();

            // con.set("key1", bytes).await?;
            // con.hset("guilds", guild.id.0, bytes).await?;

            // for c in guild.channels.iter() {
            //     let bin = bincode::serialize(&c).unwrap();
            //     con.hset("channels", c.id().0, bin).await?;
            // }

            // let items: Vec<(u64, Vec<u8>)> = guild
            //     .channels
            //     .iter()
            //     .map(|c| (c.id().0.into(), bincode::serialize(&c).unwrap()))
            //     .collect();

            // con.hset_multiple("channels", &items).await?;

            // println!("{:?}", guild.channels);
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
