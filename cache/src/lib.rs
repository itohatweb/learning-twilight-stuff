use std::ops::Deref;

use crate::config::ResourceType;
use config::Config;
use deadpool_redis::{
    redis::{cmd, AsyncCommands, RedisError},
    Config as DeadpoolConfig, Connection, Pool, Runtime,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use twilight_model::{
    channel::GuildChannel,
    gateway::{event::Event, payload::incoming::GuildCreate},
    id::{ChannelId, GuildId},
};

#[tokio::main]
async fn main() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Human {
        age: u32,
        name: String,
    }

    // let mut buf = Vec::new();
    let val = Human {
        age: 42,
        name: "John".into(),
    };

    let c: Cache<String, Human> = Cache::new("redis://127.0.0.1", "humans".into());

    c.insert("I".into(), val).await.unwrap();

    let g = c.get("I".into()).await.unwrap();

    println!("{:?}", g);

    let d = c.delete("I dont exist".into()).await.unwrap();

    println!("deleted: {:?}", d);
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Was not able to get redids db pool")]
    FailedToGetPool,
    #[error("Redis command error")]
    RedisError,
}

impl From<deadpool_redis::PoolError> for CacheError {
    fn from(_: deadpool_redis::PoolError) -> Self {
        Self::FailedToGetPool
    }
}
impl From<RedisError> for CacheError {
    fn from(_: RedisError) -> Self {
        Self::RedisError
    }
}

pub struct Cache<K, V>
where
    K: deadpool_redis::redis::ToRedisArgs + std::marker::Sync + std::marker::Send,
    V: DeserializeOwned,
{
    name: String,
    pool: Pool,
    key_type: std::marker::PhantomData<K>,
    value_type: std::marker::PhantomData<V>,
}

impl<K, V> Cache<K, V>
where
    K: deadpool_redis::redis::ToRedisArgs + std::marker::Sync + std::marker::Send,
    V: DeserializeOwned,
{
    pub fn new(connection_str: &str, map_name: String) -> Cache<K, V> {
        let cfg = DeadpoolConfig::from_url(connection_str);
        let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

        Self {
            name: map_name,
            pool,
            key_type: std::marker::PhantomData,
            value_type: std::marker::PhantomData,
        }
    }

    pub async fn insert<T>(&self, key: K, item: T) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let mut con = self.get_con().await?;

        let pack = rmp_serde::to_vec(&item).unwrap();

        con.hset(self.name.clone(), key, pack).await?;

        Ok(())
    }

    pub async fn insert_multiple<T>(&self, items: Vec<(K, T)>) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let mut con = self.get_con().await?;

        let packs = items
            .into_iter()
            .map(|c| (c.0, rmp_serde::to_vec(&c.1).unwrap()))
            .collect::<Vec<(K, Vec<u8>)>>();

        con.hset_multiple(self.name.clone(), &packs).await?;

        Ok(())
    }

    pub async fn get(&self, key: K) -> Result<V, CacheError> {
        let mut con = self.get_con().await?;

        let value: Vec<u8> = cmd("HGET")
            .arg(self.name.clone())
            .arg(key)
            .query_async(&mut con)
            .await?;

        let dec = rmp_serde::from_read(&*value).unwrap();

        Ok(dec)
    }

    pub async fn size(&self) -> Result<usize, CacheError> {
        let mut con = self.get_con().await?;

        let length: usize = con.hlen(self.name.clone()).await?;

        Ok(length)
    }

    pub async fn delete(&self, key: K) -> Result<bool, CacheError> {
        let mut con = self.get_con().await?;

        let del = con.hdel(self.name.clone(), key).await?;

        Ok(del)
    }

    async fn get_con(&self) -> Result<Connection, CacheError> {
        Ok(self.pool.get().await?)
    }
}

type Snowflake = u64;

#[derive(Debug, Deserialize, Serialize)]
pub struct GuildResource<T> {
    guild_id: GuildId,
    value: T,
}

impl<T> GuildResource<T> {
    /// ID of the guild associated with the resource.
    pub const fn guild_id(&self) -> GuildId {
        self.guild_id
    }

    /// Immutable reference to the resource's value.
    pub const fn resource(&self) -> &T {
        &self.value
    }
}

pub struct InRedisCache {
    config: Config,

    pub channels_guild: Cache<Snowflake, GuildResource<GuildChannel>>,
    // channels_guild: DashMap<ChannelId, GuildResource<GuildChannel>>,
    // channels_private: DashMap<ChannelId, PrivateChannel>
}

impl InRedisCache {
    /// Creates a new, empty cache.
    ///
    /// # Examples
    ///
    /// Creating a new `InMemoryCache` with a custom configuration, limiting
    /// the message cache to 50 messages per channel:
    ///
    /// ```
    /// use twilight_cache_inmemory::InMemoryCache;
    ///
    /// let cache = InMemoryCache::builder().message_cache_size(50).build();
    /// ```
    pub fn new() -> Self {
        let mut config = Config::new();
        config.resource_types = ResourceType::all();

        Self {
            config,
            channels_guild: Cache::new("redis://127.0.0.1", "channels_guild".into()),
        }
    }

    /// Update the cache with an event from the gateway.
    pub async fn update(&self, value: &impl UpdateCache) {
        value.update(&self).await;
    }

    /// Determine whether the configured cache wants a specific resource to be
    /// processed.
    const fn wants(&self, resource_type: ResourceType) -> bool {
        self.config.resource_types().contains(resource_type)
    }
}

/// Implemented for dispatch events.
#[async_trait::async_trait]
pub trait UpdateCache {
    /// Updates the cache based on data contained within an event.
    // Allow this for presentation purposes in documentation.
    #[allow(unused_variables)]
    async fn update(&self, cache: &InRedisCache);
}

mod config;
mod event;

// #[async_trait::async_trait]
// impl UpdateCache for GuildCreate {
//     async fn update(&self, cache: &InRedisCache) {
//         if !cache.wants(ResourceType::CHANNEL) {
//             return;
//         }

//         cache.cache_guild_channels(self.id, self.channels).await;
//     }
// }

#[async_trait::async_trait]
impl UpdateCache for Event {
    #[allow(clippy::cognitive_complexity)]
    async fn update(&self, c: &InRedisCache) {
        use Event::*;

        match self {
            BanAdd(_) => {}
            BanRemove(_) => {}
            ChannelCreate(v) => {
                c.channels_guild.insert(v.id().get(), v).await;
            }
            GuildCreate(gc) => {
                let der = gc.deref().0.clone();
                c.cache_guild_channels(der.id, der.channels).await
            }
            _ => {} // ChannelDelete(v) => c.update(v),
                    // ChannelPinsUpdate(v) => c.update(v),
                    // ChannelUpdate(v) => c.update(v),
                    // GatewayHeartbeat(_) => {}
                    // GatewayHeartbeatAck => {}
                    // GatewayHello(_) => {}
                    // GatewayInvalidateSession(_v) => {}
                    // GatewayReconnect => {}
                    // GiftCodeUpdate => {}
                    // GuildCreate(v) => c.update(v.deref()),
                    // GuildDelete(v) => c.update(v.deref()),
                    // GuildEmojisUpdate(v) => c.update(v),
                    // GuildIntegrationsUpdate(_) => {}
                    // GuildUpdate(v) => c.update(v.deref()),
                    // IntegrationCreate(v) => c.update(v.deref()),
                    // IntegrationDelete(v) => c.update(v.deref()),
                    // IntegrationUpdate(v) => c.update(v.deref()),
                    // InteractionCreate(v) => c.update(v.deref()),
                    // InviteCreate(_) => {}
                    // InviteDelete(_) => {}
                    // MemberAdd(v) => c.update(v.deref()),
                    // MemberRemove(v) => c.update(v),
                    // MemberUpdate(v) => c.update(v.deref()),
                    // MemberChunk(v) => c.update(v),
                    // MessageCreate(v) => c.update(v.deref()),
                    // MessageDelete(v) => c.update(v),
                    // MessageDeleteBulk(v) => c.update(v),
                    // MessageUpdate(v) => c.update(v.deref()),
                    // PresenceUpdate(v) => c.update(v.deref()),
                    // PresencesReplace => {}
                    // ReactionAdd(v) => c.update(v.deref()),
                    // ReactionRemove(v) => c.update(v.deref()),
                    // ReactionRemoveAll(v) => c.update(v),
                    // ReactionRemoveEmoji(v) => c.update(v),
                    // Ready(v) => c.update(v.deref()),
                    // Resumed => {}
                    // RoleCreate(v) => c.update(v),
                    // RoleDelete(v) => c.update(v),
                    // RoleUpdate(v) => c.update(v),
                    // ShardConnected(_) => {}
                    // ShardConnecting(_) => {}
                    // ShardDisconnected(_) => {}
                    // ShardIdentifying(_) => {}
                    // ShardReconnecting(_) => {}
                    // ShardPayload(_) => {}
                    // ShardResuming(_) => {}
                    // StageInstanceCreate(v) => c.update(v),
                    // StageInstanceDelete(v) => c.update(v),
                    // StageInstanceUpdate(v) => c.update(v),
                    // ThreadCreate(v) => c.update(v),
                    // ThreadUpdate(v) => c.update(v),
                    // ThreadDelete(v) => c.update(v),
                    // ThreadListSync(v) => c.update(v),
                    // ThreadMemberUpdate(_) => {}
                    // ThreadMembersUpdate(_) => {}
                    // TypingStart(_) => {}
                    // UnavailableGuild(v) => c.update(v),
                    // UserUpdate(v) => c.update(v),
                    // VoiceServerUpdate(_) => {}
                    // VoiceStateUpdate(v) => c.update(v.deref()),
                    // WebhooksUpdate(_) => {}
        }
    }
}
