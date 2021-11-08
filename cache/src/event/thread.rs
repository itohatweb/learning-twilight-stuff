use crate::{config::ResourceType, InRedisCache, UpdateCache};
use twilight_model::{
    channel::{Channel, GuildChannel},
    gateway::payload::incoming::{ThreadCreate, ThreadDelete, ThreadListSync, ThreadUpdate},
};

#[async_trait::async_trait]
impl UpdateCache for ThreadCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        if let Channel::Guild(c) = &self.0 {
            if let Some(gid) = c.guild_id() {
                cache.cache_guild_channel(gid, c.clone()).await;
            }
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for ThreadDelete {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        if let Channel::Guild(c) = &self.0 {
            if let Some(gid) = c.guild_id() {
                cache.delete_guild_channel(gid, self.0.id()).await;
            }
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for ThreadListSync {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        let threads: Vec<GuildChannel> = self
            .threads
            .iter()
            .filter_map(|c| match &c {
                Channel::Guild(c) => Some(c.clone()),
                _ => None,
            })
            .collect();

        cache.cache_guild_channels(self.guild_id, threads).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for ThreadUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        if let Channel::Guild(c) = &self.0 {
            if let Some(gid) = c.guild_id() {
                cache.cache_guild_channel(gid, c.clone()).await;
            }
        }
    }
}
