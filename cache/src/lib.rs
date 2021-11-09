use std::{ops::Deref, rc::Rc, sync::Arc};

use crate::{config::ResourceType, model::CachedGuild};
use config::Config;
use log::{error, info};
use mobc_redis::{
    mobc::{Connection, Pool},
    redis::{self, cmd, AsyncCommands, RedisError, RedisWrite, ToRedisArgs},
    RedisConnectionManager,
};
use model::{CachedEmoji, CachedMember, CachedMessage, CachedPresence};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use twilight_model::{
    channel::{message::Sticker, Group, GuildChannel, PrivateChannel, StageInstance},
    gateway::event::Event,
    guild::{GuildIntegration, Role},
    id::{ChannelId, GuildId},
    user::User,
    voice::VoiceState,
};

// #[tokio::main]
// async fn main() {
//     #[derive(Debug, PartialEq, Deserialize, Serialize)]
//     struct Human {
//         age: u32,
//         name: String,
//     }

//     // let mut buf = Vec::new();
//     let val = Human {
//         age: 42,
//         name: "John".into(),
//     };

//     let c: Cache<String, Human> = Cache::new("redis://127.0.0.1", "humans".into());

//     c.insert("I".into(), val).await.unwrap();

//     let g = c.get("I".into()).await.unwrap();

//     println!("{:?}", g);

//     let d = c.delete("I dont exist".into()).await.unwrap();

//     println!("deleted: {:?}", d);
// }

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Was not able to get redids db pool")]
    FailedToGetPool(mobc_redis::mobc::Error<RedisError>),
    #[error("Redis command error")]
    RedisError,
    #[error("Decode error: {0}")]
    DecodeError(#[from] rmp_serde::decode::Error),
    #[error("Encode error: {0}")]
    EncodeError(#[from] rmp_serde::encode::Error),
}

impl From<mobc_redis::mobc::Error<RedisError>> for CacheError {
    fn from(err: mobc_redis::mobc::Error<RedisError>) -> Self {
        Self::FailedToGetPool(err)
    }
}

impl From<RedisError> for CacheError {
    fn from(_: RedisError) -> Self {
        Self::RedisError
    }
}

pub struct RedisPool(Pool<RedisConnectionManager>);

impl RedisPool {
    pub fn new(connection_str: &str) -> Self {
        let client = redis::Client::open(connection_str).unwrap();
        let manager = RedisConnectionManager::new(client);
        let pool = Pool::builder().max_open(200).build(manager);

        Self(pool)
    }

    async fn get_con(&self) -> Option<Connection<RedisConnectionManager>> {
        match self.0.get().await {
            Ok(con) => Some(con),
            Err(_) => None,
        }
    }
}

pub struct RedisHashMapCache<K, V>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
{
    name: String,
    pool: Pool<RedisConnectionManager>,
    key_type: std::marker::PhantomData<K>,
    value_type: std::marker::PhantomData<V>,
}

impl<K, V> RedisHashMapCache<K, V>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
{
    pub fn new(connection_str: &str, map_name: String) -> RedisHashMapCache<K, V> {
        // let client = redis::Client::open(connection_str).unwrap();
        // let manager = RedisConnectionManager::new(client);
        // let pool = Pool::builder().max_open(50).build(manager);

        let client = redis::Client::open(connection_str).unwrap();
        let manager = RedisConnectionManager::new(client);
        let pool = Pool::builder().max_open(50).build(manager);

        Self {
            name: map_name,
            pool,
            key_type: std::marker::PhantomData,
            value_type: std::marker::PhantomData,
        }
    }

    pub async fn insert(&self, key: K, item: V) -> Option<()> {
        if let Some(mut con) = self.get_con().await {
            let pack = rmp_serde::to_vec(&item).ok();

            return match pack {
                Some(pack) => con
                    .hset(self.name.clone(), self.to_vec(&key).unwrap(), pack)
                    .await
                    .ok(),
                None => None,
            };
        }

        None
    }

    pub async fn insert_multiple(&self, items: Vec<(K, V)>) -> Option<()> {
        if let Some(mut con) = self.get_con().await {
            let packs = items
                .into_iter()
                .map(|c| (self.to_vec(&c.0).unwrap(), self.to_vec(&c.1).unwrap()))
                .collect::<Vec<(Vec<u8>, Vec<u8>)>>();

            return con.hset_multiple(self.name.clone(), &packs).await.ok();
        }

        None
    }

    pub async fn get(&self, key: K) -> Option<V> {
        if let Some(mut con) = self.get_con().await {
            let value: Option<Vec<u8>> = con
                .hget(self.name.clone(), rmp_serde::to_vec(&key).unwrap())
                .await
                .ok();

            return match value {
                Some(val) => match rmp_serde::from_read(&*val).ok() {
                    Some(val) => Some(val),
                    None => None,
                },
                None => None,
            };
        }

        None
    }

    pub async fn size(&self) -> Option<usize> {
        if let Some(mut con) = self.get_con().await {
            return con.hlen(self.name.clone()).await.ok();
        }

        None
    }

    pub async fn delete(&self, key: K) -> bool {
        if let Some(mut con) = self.get_con().await {
            let del = con
                .hdel(self.name.clone(), rmp_serde::to_vec(&key).unwrap())
                .await
                .ok();

            return match del {
                Some(val) => val,
                None => false,
            };
        }

        return false;
    }

    pub async fn includes(&self, key: K) -> bool {
        if let Some(mut con) = self.get_con().await {
            let has = con
                .hexists(self.name.clone(), rmp_serde::to_vec(&key).unwrap())
                .await
                .ok();

            return match has {
                Some(val) => val,
                None => false,
            };
        }

        false
    }

    async fn get_con(&self) -> Option<Connection<RedisConnectionManager>> {
        // self.pool.get_con().await
        match self.pool.get().await {
            Ok(con) => Some(con),
            Err(_) => None,
        }
    }

    fn to_vec<T: Serialize + ?Sized>(&self, val: &T) -> Option<Vec<u8>> {
        rmp_serde::to_vec(val).ok()
    }
}

pub struct RedisSetCache<K, V>
where
    K: std::fmt::Display + std::marker::Sync + std::marker::Send,
    V: DeserializeOwned + Serialize,
{
    prefix: String,
    pool: Pool<RedisConnectionManager>,
    key_type: std::marker::PhantomData<K>,
    value_type: std::marker::PhantomData<V>,
}

impl<K, V> RedisSetCache<K, V>
where
    K: std::fmt::Display + std::marker::Sync + std::marker::Send,
    V: DeserializeOwned + Serialize,
{
    pub fn new(connection_str: &str, prefix: String) -> RedisSetCache<K, V> {
        let client = redis::Client::open(connection_str).unwrap();
        let manager = RedisConnectionManager::new(client);
        let pool = Pool::builder().max_open(50).build(manager);

        Self {
            prefix,
            pool,
            key_type: std::marker::PhantomData,
            value_type: std::marker::PhantomData,
        }
    }

    pub async fn insert(&self, key: K, item: &V) -> Result<(), CacheError> {
        let mut con = self.get_con().await?;

        let pack = rmp_serde::to_vec(&item)?;

        con.sadd(self.get_key(key), pack).await?;

        Ok(())
    }

    pub async fn insert_multiple(&self, key: K, items: Vec<V>) -> Result<(), CacheError> {
        let con = self.get_con().await?;

        let packs = items
            .into_iter()
            .map(|c| rmp_serde::to_vec(&c).unwrap())
            .collect::<Vec<Vec<u8>>>();

        // con.set_multiple(self.get_key(key), &packs).await?;
        cmd("SADD")
            .arg(self.get_key(key))
            .arg(packs)
            .query_async(&mut con.into_inner())
            .await?;

        Ok(())
    }

    pub async fn get(&self, key: K) -> Result<Vec<V>, CacheError> {
        let mut con = self.get_con().await?;

        let value: Vec<Vec<u8>> = con.smembers(self.get_key(key)).await?;

        let dec = value
            .into_iter()
            .map(|v| rmp_serde::from_read(&*v).unwrap())
            .collect::<Vec<V>>();

        Ok(dec)
    }

    pub async fn size(&self, key: K) -> Result<usize, CacheError> {
        let mut con = self.get_con().await?;

        let length: usize = con.scard(self.get_key(key)).await?;

        Ok(length)
    }

    pub async fn remove(&self, key: K, item: V) -> Result<bool, CacheError> {
        let mut con = self.get_con().await?;

        let pack = rmp_serde::to_vec(&item).unwrap();
        let del = con.srem(self.get_key(key), pack).await?;
        Ok(del)
    }

    pub async fn delete(&self, key: K) -> Result<bool, CacheError> {
        let mut con = self.get_con().await?;

        let del = con.del(self.get_key(key)).await?;

        Ok(del)
    }

    pub async fn includes(&self, key: K, item: V) -> Result<bool, CacheError> {
        let mut con = self.get_con().await?;

        // TODO: check if this is correct
        let member: bool = con
            .sismember(self.get_key(key), rmp_serde::to_vec(&item).unwrap())
            .await?;

        Ok(member)
    }

    async fn get_con(&self) -> Result<Connection<RedisConnectionManager>, CacheError> {
        // Ok(self.pool.get_con().await.unwrap())
        // match self.pool.get().await {
        //     Ok(con) => Some(con),
        //     Err(_) => None,
        // }
        Ok(self.pool.get().await.unwrap())
    }

    fn get_key(&self, key: K) -> String {
        format!("{}-{}", self.prefix, key)
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

    pub channels_guild: RedisHashMapCache<Snowflake, GuildResource<GuildChannel>>,
    pub channels_private: RedisHashMapCache<Snowflake, PrivateChannel>,
    pub channel_messages: RedisSetCache<Snowflake, Snowflake>,
    // So long as the lock isn't held across await or panic points this is fine.
    // current_user: Mutex<Option<CurrentUser>>,
    pub emojis: RedisHashMapCache<Snowflake, GuildResource<CachedEmoji>>,
    pub groups: RedisHashMapCache<Snowflake, Group>,
    pub guilds: RedisHashMapCache<Snowflake, CachedGuild>,
    pub guild_channels: RedisSetCache<Snowflake, Snowflake>,
    pub guild_emojis: RedisSetCache<Snowflake, Snowflake>,
    pub guild_integrations: RedisSetCache<Snowflake, Snowflake>,
    pub guild_members: RedisSetCache<Snowflake, Snowflake>,
    pub guild_presences: RedisSetCache<Snowflake, Snowflake>,
    pub guild_roles: RedisSetCache<Snowflake, Snowflake>,
    pub guild_stage_instances: RedisSetCache<Snowflake, Snowflake>,
    pub guild_stickers: RedisSetCache<Snowflake, Snowflake>,
    pub integrations: RedisHashMapCache<(Snowflake, Snowflake), GuildResource<GuildIntegration>>,
    pub members: RedisHashMapCache<(Snowflake, Snowflake), CachedMember>,
    pub messages: RedisHashMapCache<Snowflake, CachedMessage>,
    pub presences: RedisHashMapCache<(Snowflake, Snowflake), CachedPresence>,
    pub roles: RedisHashMapCache<Snowflake, GuildResource<Role>>,
    pub stage_instances: RedisHashMapCache<Snowflake, StageInstance>,
    pub stickers: RedisHashMapCache<Snowflake, GuildResource<Sticker>>,
    pub unavailable_guilds: RedisSetCache<String, Snowflake>,
    pub users: RedisHashMapCache<Snowflake, User>,
    pub user_guilds: RedisSetCache<Snowflake, Snowflake>,
    /// Mapping of channels and the users currently connected.
    pub voice_state_channels: RedisSetCache<Snowflake, (Snowflake, Snowflake)>,
    /// Mapping of guilds and users currently connected to its voice channels.
    pub voice_state_guilds: RedisSetCache<Snowflake, Snowflake>,
    /// Mapping of guild ID and user ID pairs to their voice states.
    pub voice_states: RedisHashMapCache<(Snowflake, Snowflake), VoiceState>,
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

        // let pool = Arc::new(RedisPool::new("redis://127.0.0.1"));

        Self {
            config,
            channels_guild: RedisHashMapCache::new("redis://127.0.0.1", "channels_guild".into()),
            channels_private: RedisHashMapCache::new(
                "redis://127.0.0.1",
                "channels_private".into(),
            ),
            emojis: RedisHashMapCache::new("redis://127.0.0.1", "emojis".into()),
            groups: RedisHashMapCache::new("redis://127.0.0.1", "groups".into()),
            guilds: RedisHashMapCache::new("redis://127.0.0.1", "guilds".into()),
            integrations: RedisHashMapCache::new("redis://127.0.0.1", "integrations".into()),
            members: RedisHashMapCache::new("redis://127.0.0.1", "members".into()),
            messages: RedisHashMapCache::new("redis://127.0.0.1", "messages".into()),
            presences: RedisHashMapCache::new("redis://127.0.0.1", "presences".into()),
            roles: RedisHashMapCache::new("redis://127.0.0.1", "roles".into()),
            stage_instances: RedisHashMapCache::new("redis://127.0.0.1", "stage_instances".into()),
            stickers: RedisHashMapCache::new("redis://127.0.0.1", "stickers".into()),
            users: RedisHashMapCache::new("redis://127.0.0.1", "users".into()),
            voice_states: RedisHashMapCache::new("redis://127.0.0.1", "voice_states".into()),
            channel_messages: RedisSetCache::new("redis://127.0.0.1", "channel_messages".into()),
            guild_channels: RedisSetCache::new("redis://127.0.0.1", "guild_channels".into()),
            guild_emojis: RedisSetCache::new("redis://127.0.0.1", "guild_emojis".into()),
            guild_integrations: RedisSetCache::new(
                "redis://127.0.0.1",
                "guild_integrations".into(),
            ),
            guild_members: RedisSetCache::new("redis://127.0.0.1", "guild_members".into()),
            guild_presences: RedisSetCache::new("redis://127.0.0.1", "guild_presences".into()),
            guild_roles: RedisSetCache::new("redis://127.0.0.1", "guild_roles".into()),
            guild_stage_instances: RedisSetCache::new(
                "redis://127.0.0.1",
                "guild_stage_instances".into(),
            ),
            guild_stickers: RedisSetCache::new("redis://127.0.0.1", "guild_stickers".into()),
            unavailable_guilds: RedisSetCache::new(
                "redis://127.0.0.1",
                "unavailable_guilds".into(),
            ),
            user_guilds: RedisSetCache::new("redis://127.0.0.1", "user_guilds".into()),
            voice_state_channels: RedisSetCache::new(
                "redis://127.0.0.1",
                "voice_state_channels".into(),
            ),
            voice_state_guilds: RedisSetCache::new(
                "redis://127.0.0.1",
                "voice_state_guilds".into(),
            ),
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
mod model;

#[async_trait::async_trait]
impl UpdateCache for Event {
    #[allow(clippy::cognitive_complexity)]
    async fn update(&self, c: &InRedisCache) {
        use Event::*;

        match self {
            BanAdd(_) => {}
            BanRemove(_) => {}
            ChannelCreate(v) => c.update(v).await,
            ChannelDelete(v) => c.update(v).await,
            ChannelPinsUpdate(v) => c.update(v).await,
            ChannelUpdate(v) => c.update(v).await,
            GatewayHeartbeat(_) => {}
            GatewayHeartbeatAck => {}
            GatewayHello(_) => {}
            GatewayInvalidateSession(_v) => {}
            GatewayReconnect => {}
            GiftCodeUpdate => {}
            GuildCreate(v) => c.update(v.deref()).await,
            GuildUpdate(v) => c.update(v.deref()).await,
            GuildDelete(v) => c.update(v.deref()).await,
            GuildEmojisUpdate(v) => c.update(v).await,
            GuildIntegrationsUpdate(_) => {}
            IntegrationCreate(v) => c.update(v.deref()).await,
            IntegrationDelete(v) => c.update(v.deref()).await,
            IntegrationUpdate(v) => c.update(v.deref()).await,
            // INTERACTION
            InteractionCreate(v) => c.update(v.deref()).await,
            InviteCreate(_) => {}
            InviteDelete(_) => {}
            MemberAdd(v) => c.update(v.deref()).await,
            MemberRemove(v) => c.update(v).await,
            MemberUpdate(v) => c.update(v.deref()).await,
            MemberChunk(v) => c.update(v).await,
            MessageCreate(v) => c.update(v.deref()).await,
            MessageDelete(v) => c.update(v).await,
            MessageDeleteBulk(v) => c.update(v).await,
            MessageUpdate(v) => c.update(v.deref()).await,
            PresenceUpdate(v) => c.update(v.deref()).await,
            PresencesReplace => {}
            ReactionAdd(v) => c.update(v.deref()).await,
            ReactionRemove(v) => c.update(v.deref()).await,
            ReactionRemoveAll(v) => c.update(v).await,
            ReactionRemoveEmoji(v) => c.update(v).await,
            // Ready(v) => c.update(v.deref()).await,
            Resumed => {}
            RoleCreate(v) => c.update(v).await,
            RoleDelete(v) => c.update(v).await,
            RoleUpdate(v) => c.update(v).await,
            ShardConnected(_) => {}
            ShardConnecting(_) => {}
            ShardDisconnected(_) => {}
            ShardIdentifying(_) => {}
            ShardReconnecting(_) => {}
            ShardPayload(_) => {}
            ShardResuming(_) => {}
            StageInstanceCreate(v) => c.update(v).await,
            StageInstanceDelete(v) => c.update(v).await,
            StageInstanceUpdate(v) => c.update(v).await,
            ThreadCreate(v) => c.update(v).await,
            ThreadUpdate(v) => c.update(v).await,
            ThreadDelete(v) => c.update(v).await,
            ThreadListSync(v) => c.update(v).await,
            ThreadMemberUpdate(_) => {}
            ThreadMembersUpdate(_) => {}
            TypingStart(_) => {}
            // UnavailableGuild(v) => c.update(v).await,
            // UserUpdate(v) => c.update(v).await,
            VoiceServerUpdate(_) => {}
            // VoiceStateUpdate(v) => c.update(v.deref()).await,
            WebhooksUpdate(_) => {}
            _ => {}
        }
    }
}
