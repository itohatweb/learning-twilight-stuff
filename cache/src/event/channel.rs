use crate::{config::ResourceType, GuildResource, InRedisCache, UpdateCache};
use log::info;
use twilight_model::{
    channel::{Channel, Group, GuildChannel, PrivateChannel},
    gateway::payload::incoming::{ChannelCreate, ChannelDelete, ChannelPinsUpdate, ChannelUpdate},
    id::{ChannelId, GuildId},
};

impl InRedisCache {
    pub(crate) async fn cache_guild_channels(
        &self,
        guild_id: GuildId,
        guild_channels: impl IntoIterator<Item = GuildChannel>,
    ) {
        let mut conv = Vec::new();
        for channel in guild_channels {
            let c = GuildResource {
                guild_id,
                value: self.replace_channels_guild_id(guild_id, channel),
            };
            // conv.add((channel.id().get(), &channel));
            conv.push((c.value.id().get(), c))
        }

        self.channels_guild.insert_multiple(conv).await;
    }

    pub(crate) async fn cache_guild_channel(&self, guild_id: GuildId, channel: GuildChannel) {
        let channel = self.replace_channels_guild_id(guild_id, channel);

        // TODO cache
        // self.guild_channels.entry(guild_id).or_default().insert(id);

        self.channels_guild
            .insert(
                channel.id().get(),
                GuildResource {
                    guild_id,
                    value: channel,
                },
            )
            .await;
    }

    fn replace_channels_guild_id(
        &self,
        guild_id: GuildId,
        mut channel: GuildChannel,
    ) -> GuildChannel {
        match channel {
            GuildChannel::Category(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
            GuildChannel::NewsThread(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
            GuildChannel::PrivateThread(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
            GuildChannel::PublicThread(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
            GuildChannel::Text(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
            GuildChannel::Voice(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
            GuildChannel::Stage(ref mut c) => {
                c.guild_id.replace(guild_id);
            }
        }

        channel
    }

    async fn cache_group(&self, group: Group) {
        self.groups.insert(group.id.get(), group).await;
    }

    async fn cache_private_channel(&self, private_channel: PrivateChannel) {
        self.channels_private
            .insert(private_channel.id.get(), private_channel)
            .await;
    }

    /// Delete a guild channel from the cache.
    ///
    /// The guild channel data itself and the channel entry in its guild's list
    /// of channels will be deleted.
    pub(crate) async fn delete_guild_channel(&self, guild_id: GuildId, channel_id: ChannelId) {
        self.channels_guild.delete(channel_id.get()).await;
        self.guild_channels
            .remove(guild_id.get(), channel_id.get())
            .await
            .ok();
    }

    async fn delete_group(&self, channel_id: ChannelId) {
        self.groups.delete(channel_id.get()).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for ChannelCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        match &self.0 {
            Channel::Group(c) => {
                todo!()
                // TODO
                // crate::upsert_item(&cache.groups, c.id, c.clone());
            }
            Channel::Guild(c) => {
                if let Some(gid) = c.guild_id() {
                    cache.cache_guild_channel(gid, c.clone()).await;
                }
            }
            Channel::Private(c) => {
                cache.cache_private_channel(c.clone()).await;
            }
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for ChannelDelete {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        match self.0 {
            Channel::Group(ref c) => {
                todo!()
                // TODO
                // cache.delete_group(c.id);
            }
            Channel::Guild(ref c) => {
                cache
                    .delete_guild_channel(c.guild_id().unwrap(), c.id())
                    .await;
            }
            Channel::Private(ref c) => {
                cache.channels_private.delete(c.id.get()).await;
            }
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for ChannelPinsUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        if self.guild_id.is_some() {
            if let Some(mut r) = cache.channels_guild.get(self.channel_id.get()).await {
                if let GuildChannel::Text(ref mut text) = r.value {
                    text.last_pin_timestamp = self.last_pin_timestamp;
                }
                cache.channels_guild.insert(self.channel_id.get(), r).await;

                return;
            }
        }

        if let Some(mut channel) = cache.channels_private.get(self.channel_id.get()).await {
            channel.last_pin_timestamp = self.last_pin_timestamp;
            cache
                .channels_private
                .insert(self.channel_id.get(), channel)
                .await;

            return;
        }

        if let Some(mut group) = cache.groups.get(self.channel_id.get()).await {
            group.last_pin_timestamp = self.last_pin_timestamp;

            cache.groups.insert(self.channel_id.get(), group).await;
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for ChannelUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        match self.0.clone() {
            Channel::Group(c) => {
                cache.cache_group(c).await;
            }
            Channel::Guild(c) => {
                if let Some(gid) = c.guild_id() {
                    cache.cache_guild_channel(gid, c).await;
                }
            }
            Channel::Private(c) => {
                cache.cache_private_channel(c).await;
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test;
//     use twilight_model::gateway::event::Event;

//     #[test]
//     fn test_channel_delete_guild() {
//         let cache = InMemoryCache::new();
//         let (guild_id, channel_id, channel) = test::guild_channel_text();

//         cache.cache_guild_channel(guild_id, channel.clone());
//         assert_eq!(1, cache.channels_guild.len());
//         assert!(cache
//             .guild_channels
//             .get(&guild_id)
//             .unwrap()
//             .contains(&channel_id));

//         cache.update(&Event::ChannelDelete(ChannelDelete(Channel::Guild(
//             channel,
//         ))));
//         assert!(cache.channels_guild.is_empty());
//         assert!(cache.guild_channels.get(&guild_id).unwrap().is_empty());
//     }

//     #[test]
//     fn test_channel_update_guild() {
//         let cache = InMemoryCache::new();
//         let (guild_id, channel_id, channel) = test::guild_channel_text();

//         cache.update(&ChannelUpdate(Channel::Guild(channel)));
//         assert_eq!(1, cache.channels_guild.len());
//         assert!(cache
//             .guild_channels
//             .get(&guild_id)
//             .unwrap()
//             .contains(&channel_id));
//     }
// }
