use crate::{config::ResourceType, InRedisCache, UpdateCache};
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
        // let guild_channels: Vec<(u64, &GuildChannel)> = &guild_channels
        //     .into_iter()
        //     .map(|c| (c.id().get(), self.replace_channels_guild_id(guild_id, c)))
        //     .collect();
        // let guild_channels: Vec<(u64, &GuildChannel)> = guild_channels
        //     .into_iter()
        //     .map(|c| (c.id().get(), self.replace_channels_guild_id(guild_id, c)))
        //     .collect();

        let mut conv = Vec::new();
        for channel in guild_channels {
            let c = self.replace_channels_guild_id(guild_id, channel);
            // conv.add((channel.id().get(), &channel));
            conv.push((c.id().get(), c))
        }

        self.channels_guild.insert_multiple(conv).await;

        // for channel in guild_channels {
        //     self.cache_guild_channel(guild_id, channel);
        // }
    }

    pub(crate) async fn cache_guild_channel(&self, guild_id: GuildId, channel: GuildChannel) {
        let channel = self.replace_channels_guild_id(guild_id, channel);

        // TODO: self.guild_channels.entry(guild_id).or_default().insert(id);

        self.channels_guild
            .insert(channel.id().get(), channel)
            .await;

        // crate::upsert_guild_item(&self.channels_guild, guild_id, id, channel);
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

    // fn cache_group(&self, group: Group) {
    //     crate::upsert_item(&self.groups, group.id, group)
    // }

    // fn cache_private_channel(&self, private_channel: PrivateChannel) {
    //     self.channels_private
    //         .insert(private_channel.id, private_channel);
    // }

    /// Delete a guild channel from the cache.
    ///
    /// The guild channel data itself and the channel entry in its guild's list
    /// of channels will be deleted.
    fn f() {}
    // pub(crate) fn delete_guild_channel(&self, channel_id: ChannelId) {
    //     if let Some((_, item)) = self.channels_guild.remove(&channel_id) {
    //         if let Some(mut guild_channels) = self.guild_channels.get_mut(&item.guild_id) {
    //             guild_channels.remove(&channel_id);
    //         }
    //     }
    // }

    // fn delete_group(&self, channel_id: ChannelId) {
    //     self.groups.remove(&channel_id);
    // }
}

#[async_trait::async_trait]
impl UpdateCache for ChannelCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::CHANNEL) {
            return;
        }

        match &self.0 {
            Channel::Group(c) => {
                // crate::upsert_item(&cache.groups, c.id, c.clone());
            }
            Channel::Guild(c) => {
                if let Some(gid) = c.guild_id() {
                    cache.cache_guild_channel(gid, c.clone());
                }
            }
            Channel::Private(c) => {
                // cache.cache_private_channel(c.clone());
            }
        }
    }
}

// impl UpdateCache for ChannelDelete {
//     fn update(&self, cache: &InMemoryCache) {
//         if !cache.wants(ResourceType::CHANNEL) {
//             return;
//         }

//         match self.0 {
//             Channel::Group(ref c) => {
//                 cache.delete_group(c.id);
//             }
//             Channel::Guild(ref c) => {
//                 cache.delete_guild_channel(c.id());
//             }
//             Channel::Private(ref c) => {
//                 cache.channels_private.remove(&c.id);
//             }
//         }
//     }
// }

// impl UpdateCache for ChannelPinsUpdate {
//     fn update(&self, cache: &InMemoryCache) {
//         if !cache.wants(ResourceType::CHANNEL) {
//             return;
//         }

//         if let Some(mut r) = cache.channels_guild.get_mut(&self.channel_id) {
//             let value = r.value_mut();

//             if let GuildChannel::Text(ref mut text) = value.value {
//                 text.last_pin_timestamp = self.last_pin_timestamp;
//             }

//             return;
//         }

//         if let Some(mut channel) = cache.channels_private.get_mut(&self.channel_id) {
//             channel.last_pin_timestamp = self.last_pin_timestamp;

//             return;
//         }

//         if let Some(mut group) = cache.groups.get_mut(&self.channel_id) {
//             group.last_pin_timestamp = self.last_pin_timestamp;
//         }
//     }
// }

// impl UpdateCache for ChannelUpdate {
//     fn update(&self, cache: &InMemoryCache) {
//         if !cache.wants(ResourceType::CHANNEL) {
//             return;
//         }

//         match self.0.clone() {
//             Channel::Group(c) => {
//                 cache.cache_group(c);
//             }
//             Channel::Guild(c) => {
//                 if let Some(gid) = c.guild_id() {
//                     cache.cache_guild_channel(gid, c);
//                 }
//             }
//             Channel::Private(c) => {
//                 cache.cache_private_channel(c);
//             }
//         }
//     }
// }

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
