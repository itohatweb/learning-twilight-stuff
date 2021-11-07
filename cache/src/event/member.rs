use crate::{config::ResourceType, model::CachedMember, InRedisCache, UpdateCache};
use std::borrow::Cow;
use twilight_model::{
    application::interaction::application_command::InteractionMember,
    gateway::payload::incoming::{MemberAdd, MemberChunk, MemberRemove, MemberUpdate},
    guild::{Member, PartialMember},
    id::{GuildId, UserId},
};

impl InRedisCache {
    pub(crate) async fn cache_members(
        &self,
        guild_id: GuildId,
        members: impl IntoIterator<Item = Member>,
    ) {
        for member in members {
            self.cache_member(guild_id, member).await;
        }
    }

    pub(crate) async fn cache_member(&self, guild_id: GuildId, member: Member) {
        let member_id = member.user.id;
        let id = (guild_id.get(), member_id.get());

        if let Some(m) = self.members.get(id).await {
            if m == member {
                return;
            }
        }

        let user_id = member.user.id;

        self.cache_user(Cow::Owned(member.user), Some(guild_id));
        let cached = CachedMember {
            deaf: Some(member.deaf),
            guild_id,
            joined_at: member.joined_at,
            mute: Some(member.mute),
            nick: member.nick,
            pending: member.pending,
            premium_since: member.premium_since,
            roles: member.roles,
            user_id,
        };
        self.members.insert(id, cached).await;
        self.guild_members
            .insert(guild_id.get(), &member_id.get())
            .await;
    }

    pub(crate) async fn cache_borrowed_partial_member(
        &self,
        guild_id: GuildId,
        member: &PartialMember,
        user_id: UserId,
    ) {
        let id = (guild_id.get(), user_id.get());

        if let Some(m) = self.members.get(id).await {
            if m == member {
                return;
            }
        }

        self.guild_members
            .insert(guild_id.get(), &user_id.get())
            .await;

        let cached = CachedMember {
            deaf: Some(member.deaf),
            guild_id,
            joined_at: member.joined_at.to_owned(),
            mute: Some(member.mute),
            nick: member.nick.to_owned(),
            pending: false,
            premium_since: None,
            roles: member.roles.to_owned(),
            user_id,
        };
        self.members.insert(id, cached).await;
    }

    pub(crate) async fn cache_borrowed_interaction_member(
        &self,
        guild_id: GuildId,
        member: &InteractionMember,
    ) {
        let id = (guild_id.get(), member.id.get());

        let (deaf, mute) = match self.members.get(id).await {
            Some(m) if m == member => return,
            Some(m) => (m.deaf(), m.mute()),
            _ => (None, None),
        };

        self.guild_members
            .insert(guild_id.get(), &member.id.get())
            .await;

        let cached = CachedMember {
            deaf,
            guild_id,
            joined_at: member.joined_at.to_owned(),
            mute,
            nick: member.nick.to_owned(),
            pending: false,
            premium_since: member.premium_since.to_owned(),
            roles: member.roles.to_owned(),
            user_id: member.id,
        };

        self.members.insert(id, cached).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for MemberAdd {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::MEMBER) {
            return;
        }

        cache.cache_member(self.guild_id, self.0.clone()).await;

        cache
            .guild_members
            .insert(self.guild_id.get(), &self.0.user.id.get())
            .await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for MemberChunk {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::MEMBER) {
            return;
        }

        if self.members.is_empty() {
            return;
        }

        cache
            .cache_members(self.guild_id, self.members.clone())
            .await;

        cache
            .guild_members
            .insert_multiple(
                self.guild_id.get(),
                self.members
                    .iter()
                    .map(|member| member.user.id.get())
                    .collect(),
            )
            .await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for MemberRemove {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::MEMBER) {
            return;
        }

        cache
            .members
            .delete((self.guild_id.get(), self.user.id.get()))
            .await;

        cache
            .guild_members
            .remove(self.guild_id.get(), self.user.id.get())
            .await;

        // TODO: optimize this
        if let Some(mut user_guilds) = cache.user_guilds.get(self.user.id.get()).await.ok() {
            if let Some(index) = user_guilds.iter().position(|id| id == &self.guild_id.get()) {
                user_guilds.remove(index);
            }

            if user_guilds.is_empty() {
                cache.users.delete(self.user.id.get()).await;
                cache.user_guilds.delete(self.user.id.get()).await;
            } else {
                cache
                    .user_guilds
                    .remove(self.user.id.get(), self.guild_id.get())
                    .await;
            }
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for MemberUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::MEMBER) {
            return;
        }

        let mut member = match cache
            .members
            .get((self.guild_id.get(), self.user.id.get()))
            .await
        {
            Some(member) => member,
            None => return,
        };

        member.deaf = self.deaf.or_else(|| member.deaf());
        member.mute = self.mute.or_else(|| member.mute());
        member.nick = self.nick.clone();
        member.roles = self.roles.clone();
        member.joined_at.replace(self.joined_at);
        member.pending = self.pending;

        cache
            .members
            .insert((self.guild_id.get(), self.user.id.get()), member)
            .await;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test;

//     #[test]
//     fn test_cache_guild_member() {
//         let cache = InMemoryCache::new();

//         // Single inserts
//         {
//             let guild_1_user_ids = (1..=10)
//                 .map(|n| UserId::new(n).expect("non zero"))
//                 .collect::<Vec<_>>();
//             let guild_1_members = guild_1_user_ids
//                 .iter()
//                 .copied()
//                 .map(|id| test::member(id, GuildId::new(1).expect("non zero")))
//                 .collect::<Vec<_>>();

//             for member in guild_1_members {
//                 cache.cache_member(GuildId::new(1).expect("non zero"), member);
//             }

//             // Check for the cached guild members ids
//             let cached_roles = cache
//                 .guild_members(GuildId::new(1).expect("non zero"))
//                 .unwrap();
//             assert_eq!(cached_roles.len(), guild_1_user_ids.len());
//             assert!(guild_1_user_ids.iter().all(|id| cached_roles.contains(id)));

//             // Check for the cached members
//             assert!(guild_1_user_ids.iter().all(|id| cache
//                 .member(GuildId::new(1).expect("non zero"), *id)
//                 .is_some()));

//             // Check for the cached users
//             assert!(guild_1_user_ids.iter().all(|id| cache.user(*id).is_some()));
//         }

//         // Bulk inserts
//         {
//             let guild_2_user_ids = (1..=10)
//                 .map(|n| UserId::new(n).expect("non zero"))
//                 .collect::<Vec<_>>();
//             let guild_2_members = guild_2_user_ids
//                 .iter()
//                 .copied()
//                 .map(|id| test::member(id, GuildId::new(2).expect("non zero")))
//                 .collect::<Vec<_>>();
//             cache.cache_members(GuildId::new(2).expect("non zero"), guild_2_members);

//             // Check for the cached guild members ids
//             let cached_roles = cache
//                 .guild_members(GuildId::new(1).expect("non zero"))
//                 .unwrap();
//             assert_eq!(cached_roles.len(), guild_2_user_ids.len());
//             assert!(guild_2_user_ids.iter().all(|id| cached_roles.contains(id)));

//             // Check for the cached members
//             assert!(guild_2_user_ids.iter().copied().all(|id| cache
//                 .member(GuildId::new(1).expect("non zero"), id)
//                 .is_some()));

//             // Check for the cached users
//             assert!(guild_2_user_ids.iter().all(|id| cache.user(*id).is_some()));
//         }
//     }

//     #[test]
//     fn test_cache_user_guild_state() {
//         let user_id = UserId::new(2).expect("non zero");
//         let cache = InMemoryCache::new();
//         cache.cache_user(
//             Cow::Owned(test::user(user_id)),
//             Some(GuildId::new(1).expect("non zero")),
//         );

//         // Test the guild's ID is the only one in the user's set of guilds.
//         {
//             let user_guilds = cache.user_guilds.get(&user_id).unwrap();
//             assert!(user_guilds.contains(&GuildId::new(1).expect("non zero")));
//             assert_eq!(1, user_guilds.len());
//         }

//         // Test that a second guild will cause 2 in the set.
//         cache.cache_user(
//             Cow::Owned(test::user(user_id)),
//             Some(GuildId::new(3).expect("non zero")),
//         );

//         {
//             let user_guilds = cache.user_guilds.get(&user_id).unwrap();
//             assert!(user_guilds.contains(&GuildId::new(3).expect("non zero")));
//             assert_eq!(2, user_guilds.len());
//         }

//         // Test that removing a user from a guild will cause the ID to be
//         // removed from the set, leaving the other ID.
//         cache.update(&MemberRemove {
//             guild_id: GuildId::new(3).expect("non zero"),
//             user: test::user(user_id),
//         });

//         {
//             let user_guilds = cache.user_guilds.get(&user_id).unwrap();
//             assert!(!user_guilds.contains(&GuildId::new(3).expect("non zero")));
//             assert_eq!(1, user_guilds.len());
//         }

//         // Test that removing the user from its last guild removes the user's
//         // entry.
//         cache.update(&MemberRemove {
//             guild_id: GuildId::new(1).expect("non zero"),
//             user: test::user(user_id),
//         });
//         assert!(!cache.users.contains_key(&user_id));
//     }
// }
