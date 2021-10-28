use dotenv::dotenv;
use futures::stream::StreamExt;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::{callback::InteractionResponse, interaction::Interaction},
    gateway::Intents,
};
use twilight_util::builder::CallbackDataBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Load the .env file and just ignore any errors
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN")?;

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(
        token.to_owned(),
        Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES,
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
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            http.create_message(msg.channel_id)
                .content("Pong!")?
                .exec()
                .await?;

            println!("---- Message event: {:?}", msg);
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        Event::VoiceStateUpdate(vsu) => {
            println!("vsu: {:?}", vsu);
        }
        Event::InteractionCreate(interaction) => {
            if let Interaction::ApplicationCommand(cmd) = interaction.0 {
                http.interaction_callback(
                    cmd.id,
                    &cmd.token,
                    &InteractionResponse::ChannelMessageWithSource(
                        CallbackDataBuilder::new()
                            .content("foo bar baz".into())
                            .build(),
                    ),
                )
                .exec()
                .await?;
            }
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
