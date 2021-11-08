use crate::{
    config::ResourceType,
    model::{CachedGuild, CachedPresence},
    InRedisCache, RedisHashMapCache, RedisSetCache, UpdateCache,
};
use std::{collections::HashSet, hash::Hash};
use twilight_model::{
    gateway::payload::incoming::{GuildCreate, GuildDelete, GuildUpdate},
    guild::Guild,
    id::GuildId,
};

impl InRedisCache {
    async fn cache_guild(&self, guild: Guild) {
        // The map and set creation needs to occur first, so caching states and
        // objects always has a place to put them.
        if self.wants(ResourceType::CHANNEL) {
            self.cache_guild_channels(guild.id, guild.channels).await;
            self.cache_guild_channels(guild.id, guild.threads).await;
        }

        if self.wants(ResourceType::EMOJI) {
            self.cache_emojis(guild.id, guild.emojis).await;
        }

        if self.wants(ResourceType::MEMBER) {
            self.cache_members(guild.id, guild.members).await;
        }

        if self.wants(ResourceType::PRESENCE) {
            self.cache_presences(
                guild.id,
                guild.presences.into_iter().map(CachedPresence::from),
            )
            .await;
        }

        if self.wants(ResourceType::ROLE) {
            self.cache_roles(guild.id, guild.roles).await;
        }

        if self.wants(ResourceType::STICKER) {
            // self.cache_stickers(guild.id, guild.stickers).await;
        }

        if self.wants(ResourceType::VOICE_STATE) {
            // self.cache_voice_states(guild.voice_states).await;
        }

        if self.wants(ResourceType::STAGE_INSTANCE) {
            self.cache_stage_instances(guild.id, guild.stage_instances)
                .await;
        }

        let guild = CachedGuild {
            id: guild.id,
            afk_channel_id: guild.afk_channel_id,
            afk_timeout: guild.afk_timeout,
            application_id: guild.application_id,
            banner: guild.banner,
            default_message_notifications: guild.default_message_notifications,
            description: guild.description,
            discovery_splash: guild.discovery_splash,
            explicit_content_filter: guild.explicit_content_filter,
            features: guild.features,
            icon: guild.icon,
            joined_at: guild.joined_at,
            large: guild.large,
            max_members: guild.max_members,
            max_presences: guild.max_presences,
            member_count: guild.member_count,
            mfa_level: guild.mfa_level,
            name: guild.name,
            nsfw_level: guild.nsfw_level,
            owner: guild.owner,
            owner_id: guild.owner_id,
            permissions: guild.permissions,
            preferred_locale: guild.preferred_locale,
            premium_subscription_count: guild.premium_subscription_count,
            premium_tier: guild.premium_tier,
            rules_channel_id: guild.rules_channel_id,
            splash: guild.splash,
            system_channel_id: guild.system_channel_id,
            system_channel_flags: guild.system_channel_flags,
            unavailable: guild.unavailable,
            verification_level: guild.verification_level,
            vanity_url_code: guild.vanity_url_code,
            widget_channel_id: guild.widget_channel_id,
            widget_enabled: guild.widget_enabled,
        };

        self.unavailable_guilds
            .remove("unavailable_guilds".into(), guild.id().get())
            .await;
        self.guilds.insert(guild.id().get(), guild).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for GuildCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::GUILD) {
            return;
        }

        cache.cache_guild(self.0.clone()).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for GuildDelete {
    async fn update(&self, cache: &InRedisCache) {
        // TODO: WHATS THIS
        async fn remove_ids<Hv>(
            target: &RedisHashMapCache<u64, Hv>,
            // guild_map: &DashMap<GuildId, HashSet<T>>,
            // container: &DashMap<T, U>,
            from: &RedisSetCache<u64, u64>,
            guild_id: u64,
        ) where
            Hv: serde::de::DeserializeOwned + serde::Serialize,
        {
            if let Some(res) = from.get(guild_id).await.ok() {
                for cid in res {
                    target.delete(cid).await;
                }
            }
            // if let Some((_, ids)) = guild_map.remove(&guild_id) {
            //     for id in ids {
            //         container.remove(&id);
            //     }
            // }
        }

        if !cache.wants(ResourceType::GUILD) {
            return;
        }

        let id = self.id.get();

        cache.guilds.delete(id).await;

        if cache.wants(ResourceType::CHANNEL) {
            remove_ids(&cache.channels_guild, &cache.guild_channels, id).await;
        }

        if cache.wants(ResourceType::EMOJI) {
            remove_ids(&cache.emojis, &cache.guild_emojis, id).await;
        }

        if cache.wants(ResourceType::ROLE) {
            remove_ids(&cache.roles, &cache.guild_roles, id).await;
        }

        if cache.wants(ResourceType::STICKER) {
            remove_ids(&cache.stickers, &cache.guild_stickers, id).await;
        }

        if cache.wants(ResourceType::VOICE_STATE) {
            // Clear out a guilds voice states when a guild leaves
            cache.voice_state_guilds.delete(id).await;
        }

        if cache.wants(ResourceType::MEMBER) {
            if let Ok(members) = cache.guild_members.get(id).await {
                for mid in members {
                    cache.members.delete((id, mid)).await;
                }
            }

            cache.guild_members.delete(id).await;
        }

        if cache.wants(ResourceType::PRESENCE) {
            if let Ok(presences) = cache.guild_presences.get(id).await {
                for mid in presences {
                    cache.presences.delete((id, mid)).await;
                }
            }

            cache.guild_presences.delete(id).await;
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for GuildUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::GUILD) {
            return;
        }

        if let Some(mut guild) = cache.guilds.get(self.0.id.get()).await {
            guild.afk_channel_id = self.afk_channel_id;
            guild.afk_timeout = self.afk_timeout;
            guild.banner = self.banner.clone();
            guild.default_message_notifications = self.default_message_notifications;
            guild.description = self.description.clone();
            guild.features = self.features.clone();
            guild.icon = self.icon.clone();
            guild.max_members = self.max_members;
            guild.max_presences = Some(self.max_presences.unwrap_or(25000));
            guild.mfa_level = self.mfa_level;
            guild.name = self.name.clone();
            guild.nsfw_level = self.nsfw_level;
            guild.owner = self.owner;
            guild.owner_id = self.owner_id;
            guild.permissions = self.permissions;
            guild.preferred_locale = self.preferred_locale.clone();
            guild.premium_tier = self.premium_tier;
            guild
                .premium_subscription_count
                .replace(self.premium_subscription_count.unwrap_or_default());
            guild.splash = self.splash.clone();
            guild.system_channel_id = self.system_channel_id;
            guild.verification_level = self.verification_level;
            guild.vanity_url_code = self.vanity_url_code.clone();
            guild.widget_channel_id = self.widget_channel_id;
            guild.widget_enabled = self.widget_enabled;

            cache.guilds.insert(self.0.id.get(), guild);
        };
    }
}

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use super::*;
//     use twilight_model::{
//         channel::{
//             thread::{AutoArchiveDuration, PublicThread, ThreadMember, ThreadMetadata},
//             ChannelType, GuildChannel, TextChannel,
//         },
//         datetime::{Timestamp, TimestampParseError},
//         guild::{
//             DefaultMessageNotificationLevel, ExplicitContentFilter, MfaLevel, NSFWLevel,
//             PartialGuild, Permissions, PremiumTier, SystemChannelFlags, VerificationLevel,
//         },
//         id::{ChannelId, GuildId, UserId},
//     };

//     #[test]
//     fn test_guild_create_channels_have_guild_ids() -> Result<(), TimestampParseError> {
//         const DATETIME: &str = "2021-09-19T14:17:32.000000+00:00";

//         let timestamp = Timestamp::from_str(DATETIME)?;

//         let channels = Vec::from([GuildChannel::Text(TextChannel {
//             id: ChannelId::new(111).expect("non zero"),
//             guild_id: None,
//             kind: ChannelType::GuildText,
//             last_message_id: None,
//             last_pin_timestamp: None,
//             name: "guild channel with no guild id".to_owned(),
//             nsfw: true,
//             permission_overwrites: Vec::new(),
//             parent_id: None,
//             position: 1,
//             rate_limit_per_user: None,
//             topic: None,
//         })]);

//         let threads = Vec::from([GuildChannel::PublicThread(PublicThread {
//             id: ChannelId::new(222).expect("non zero"),
//             default_auto_archive_duration: None,
//             guild_id: None,
//             kind: ChannelType::GuildPublicThread,
//             last_message_id: None,
//             message_count: 0,
//             name: "guild thread with no guild id".to_owned(),
//             owner_id: None,
//             parent_id: None,
//             rate_limit_per_user: None,
//             member_count: 0,
//             thread_metadata: ThreadMetadata {
//                 archived: false,
//                 auto_archive_duration: AutoArchiveDuration::Hour,
//                 archive_timestamp: timestamp,
//                 invitable: None,
//                 locked: false,
//             },
//             member: Some(ThreadMember {
//                 flags: 0,
//                 id: Some(ChannelId::new(1).expect("non zero")),
//                 join_timestamp: timestamp,
//                 member: None,
//                 presence: None,
//                 user_id: Some(UserId::new(2).expect("non zero")),
//             }),
//         })]);

//         let guild = Guild {
//             id: GuildId::new(123).expect("non zero"),
//             afk_channel_id: None,
//             afk_timeout: 300,
//             application_id: None,
//             banner: None,
//             channels,
//             default_message_notifications: DefaultMessageNotificationLevel::Mentions,
//             description: None,
//             discovery_splash: None,
//             emojis: Vec::new(),
//             explicit_content_filter: ExplicitContentFilter::AllMembers,
//             features: vec![],
//             icon: None,
//             joined_at: Some(Timestamp::from_secs(1_632_072_645).expect("non zero")),
//             large: false,
//             max_members: Some(50),
//             max_presences: Some(100),
//             member_count: Some(25),
//             members: Vec::new(),
//             mfa_level: MfaLevel::Elevated,
//             name: "this is a guild".to_owned(),
//             nsfw_level: NSFWLevel::AgeRestricted,
//             owner: Some(false),
//             owner_id: UserId::new(456).expect("non zero"),
//             permissions: Some(Permissions::SEND_MESSAGES),
//             preferred_locale: "en-GB".to_owned(),
//             premium_subscription_count: Some(0),
//             premium_tier: PremiumTier::None,
//             presences: Vec::new(),
//             roles: Vec::new(),
//             splash: None,
//             stage_instances: Vec::new(),
//             stickers: Vec::new(),
//             system_channel_id: None,
//             system_channel_flags: SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATIONS,
//             rules_channel_id: None,
//             threads,
//             unavailable: false,
//             verification_level: VerificationLevel::VeryHigh,
//             voice_states: Vec::new(),
//             vanity_url_code: None,
//             widget_channel_id: None,
//             widget_enabled: None,
//             max_video_channel_users: None,
//             approximate_member_count: None,
//             approximate_presence_count: None,
//         };

//         let cache = InMemoryCache::new();
//         cache.cache_guild(guild);

//         let channel = cache
//             .guild_channel(ChannelId::new(111).expect("non zero"))
//             .unwrap();

//         let thread = cache
//             .guild_channel(ChannelId::new(222).expect("non zero"))
//             .unwrap();

//         // The channel was given to the cache without a guild ID, but because
//         // it's part of a guild create, the cache can automatically attach the
//         // guild ID to it. So now, the channel's guild ID is present with the
//         // correct value.
//         match channel.resource() {
//             GuildChannel::Text(ref c) => {
//                 assert_eq!(Some(GuildId::new(123).expect("non zero")), c.guild_id);
//             }
//             _ => panic!("{:?}", channel),
//         }

//         match thread.resource() {
//             GuildChannel::PublicThread(ref c) => {
//                 assert_eq!(Some(GuildId::new(123).expect("non zero")), c.guild_id);
//             }
//             _ => panic!("{:?}", channel),
//         }

//         Ok(())
//     }

//     #[test]
//     fn test_guild_update() {
//         let cache = InMemoryCache::new();
//         let guild = Guild {
//             afk_channel_id: None,
//             afk_timeout: 0,
//             application_id: None,
//             approximate_member_count: None,
//             approximate_presence_count: None,
//             banner: None,
//             channels: Vec::new(),
//             default_message_notifications: DefaultMessageNotificationLevel::Mentions,
//             description: None,
//             discovery_splash: None,
//             emojis: Vec::new(),
//             explicit_content_filter: ExplicitContentFilter::None,
//             features: Vec::new(),
//             icon: None,
//             id: GuildId::new(1).expect("non zero"),
//             joined_at: None,
//             large: false,
//             max_members: None,
//             max_presences: None,
//             max_video_channel_users: None,
//             member_count: None,
//             members: Vec::new(),
//             mfa_level: MfaLevel::None,
//             name: "test".to_owned(),
//             nsfw_level: NSFWLevel::Default,
//             owner_id: UserId::new(1).expect("non zero"),
//             owner: None,
//             permissions: None,
//             preferred_locale: "en_us".to_owned(),
//             premium_subscription_count: None,
//             premium_tier: PremiumTier::None,
//             presences: Vec::new(),
//             roles: Vec::new(),
//             rules_channel_id: None,
//             splash: None,
//             stage_instances: Vec::new(),
//             stickers: Vec::new(),
//             system_channel_flags: SystemChannelFlags::empty(),
//             system_channel_id: None,
//             threads: Vec::new(),
//             unavailable: false,
//             vanity_url_code: None,
//             verification_level: VerificationLevel::VeryHigh,
//             voice_states: Vec::new(),
//             widget_channel_id: None,
//             widget_enabled: None,
//         };

//         cache.update(&GuildCreate(guild.clone()));

//         let mutation = PartialGuild {
//             id: guild.id,
//             afk_channel_id: guild.afk_channel_id,
//             afk_timeout: guild.afk_timeout,
//             application_id: guild.application_id,
//             banner: guild.banner,
//             default_message_notifications: guild.default_message_notifications,
//             description: guild.description,
//             discovery_splash: guild.discovery_splash,
//             emojis: guild.emojis,
//             explicit_content_filter: guild.explicit_content_filter,
//             features: guild.features,
//             icon: guild.icon,
//             max_members: guild.max_members,
//             max_presences: guild.max_presences,
//             member_count: guild.member_count,
//             mfa_level: guild.mfa_level,
//             name: "test2222".to_owned(),
//             nsfw_level: guild.nsfw_level,
//             owner_id: UserId::new(2).expect("non zero"),
//             owner: guild.owner,
//             permissions: guild.permissions,
//             preferred_locale: guild.preferred_locale,
//             premium_subscription_count: guild.premium_subscription_count,
//             premium_tier: guild.premium_tier,
//             roles: guild.roles,
//             rules_channel_id: guild.rules_channel_id,
//             splash: guild.splash,
//             system_channel_flags: guild.system_channel_flags,
//             system_channel_id: guild.system_channel_id,
//             verification_level: guild.verification_level,
//             vanity_url_code: guild.vanity_url_code,
//             widget_channel_id: guild.widget_channel_id,
//             widget_enabled: guild.widget_enabled,
//         };

//         cache.update(&GuildUpdate(mutation.clone()));

//         assert_eq!(cache.guild(guild.id).unwrap().name, mutation.name);
//         assert_eq!(cache.guild(guild.id).unwrap().owner_id, mutation.owner_id);
//         assert_eq!(cache.guild(guild.id).unwrap().id, mutation.id);
//     }
// }
