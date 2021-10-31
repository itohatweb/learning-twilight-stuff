use bot_test::{
    commands::exec_command,
    types::{RCache, TwHttpClient},
};
use dotenv::dotenv;
use futures::stream::StreamExt;
use parking_lot::RwLock;
use std::{borrow::BorrowMut, env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::Interaction, channel::Channel, gateway::Intents, id::UserId,
};

use simple_process_stats::ProcessStats;

// TODO: look at this cool thing when its finished https://github.com/baptiste0928/twilight-interactions

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Load the .env file and just ignore any errors
    dotenv().ok();
    env_logger::init();

    info!("Starting up");

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
    let cache: RCache = std::sync::Arc::new(RwLock::new(
        InMemoryCache::builder()
            .resource_types(ResourceType::all())
            .build(),
    ));

    let new_cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(ResourceType::all())
            .build(),
    );

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        new_cache.update(&event);

        tokio::spawn(handle_event(
            shard_id,
            event,
            Arc::clone(&http),
            new_cache.clone(),
        ));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: TwHttpClient,
    cache: Arc<InMemoryCache>,
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

            let embed = EmbedBuilder::new()
                .description("Current statistics of the bot:")
                .field(EmbedFieldBuilder::new(
                    "Cached Stuff:",
                    format!(
                        "guilds: {}\nchannels: {}\nmessages: {}\nmembers: {}\nvoice states: {}",
                        guild_count, channel_count, message_count, member_count, voice_states_count
                    ),
                ))
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
            // println!("---- Message event: {:?}", msg);
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        Event::VoiceStateUpdate(vsu) => {
            // println!("vsu: {:?}", vsu);
        }
        Event::InteractionCreate(interaction) => {
            if let Interaction::ApplicationCommand(command) = interaction.0 {
                // exec_command(http, &command, cache).await?;
            }
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}

pub mod cache_inmemory {
    use std::{ops::Deref, sync::Arc};

    pub use twilight_cache_inmemory::*;

    #[derive(Clone)]
    pub struct CloneableInMemoryCache(pub Arc<InMemoryCache>);

    impl CloneableInMemoryCache {
        pub fn new(cache: InMemoryCache) -> Self {
            Self(Arc::new(cache))
        }
    }

    impl Deref for CloneableInMemoryCache {
        type Target = InMemoryCache;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
