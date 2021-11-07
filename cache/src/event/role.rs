use crate::{config::ResourceType, GuildResource, InRedisCache, UpdateCache};
use twilight_model::{
    gateway::payload::incoming::{RoleCreate, RoleDelete, RoleUpdate},
    guild::Role,
    id::{GuildId, RoleId},
};

impl InRedisCache {
    pub(crate) async fn cache_roles(
        &self,
        guild_id: GuildId,
        roles: impl IntoIterator<Item = Role>,
    ) {
        let mut roles_to_cache = vec![];
        let mut guild_roles = vec![];

        for role in roles {
            guild_roles.push(role.id.get());
            roles_to_cache.push((
                role.id.get(),
                GuildResource {
                    guild_id,
                    value: role,
                },
            ))
            // self.cache_role(guild_id, role);
        }

        self.roles.insert_multiple(roles_to_cache).await;
        self.guild_roles
            .insert_multiple(guild_id.get(), guild_roles)
            .await;
    }

    async fn cache_role(&self, guild_id: GuildId, role: Role) {
        // Insert the role into the guild_roles map
        self.guild_roles
            .insert(guild_id.get(), &role.id.get())
            .await;

        // Insert the role into the all roles map
        self.roles.insert(
            guild_id.get(),
            GuildResource {
                guild_id,
                value: role,
            },
        );
    }

    async fn delete_role(&self, guild_id: GuildId, role_id: RoleId) {
        if true == self.roles.delete(role_id.get()).await {
            self.guild_roles.remove(guild_id.get(), role_id.get()).await;
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for RoleCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::ROLE) {
            return;
        }

        cache.cache_role(self.guild_id, self.role.clone()).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for RoleDelete {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::ROLE) {
            return;
        }

        cache.delete_role(self.guild_id, self.role_id).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for RoleUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::ROLE) {
            return;
        }

        cache.cache_role(self.guild_id, self.role.clone()).await;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test;

//     #[test]
//     fn test_insert_role_on_event() {
//         let cache = InMemoryCache::new();

//         cache.update(&RoleCreate {
//             guild_id: GuildId::new(1).expect("non zero"),
//             role: test::role(RoleId::new(2).expect("non zero")),
//         });

//         {
//             assert_eq!(
//                 1,
//                 cache
//                     .guild_roles
//                     .get(&GuildId::new(1).expect("non zero"))
//                     .unwrap()
//                     .len()
//             );
//             assert_eq!(1, cache.roles.len());

//             assert_eq!(
//                 "test".to_string(),
//                 cache.role(RoleId::new(2).expect("non zero")).unwrap().name
//             );
//         }
//     }

//     #[test]
//     fn test_cache_role() {
//         let cache = InMemoryCache::new();

//         // Single inserts
//         {
//             // The role ids for the guild with id 1
//             let guild_1_role_ids = (1..=10)
//                 .map(|n| RoleId::new(n).expect("non zero"))
//                 .collect::<Vec<_>>();
//             // Map the role ids to a test role
//             let guild_1_roles = guild_1_role_ids
//                 .iter()
//                 .copied()
//                 .map(test::role)
//                 .collect::<Vec<_>>();
//             // Cache all the roles using cache role
//             for role in guild_1_roles.clone() {
//                 cache.cache_role(GuildId::new(1).expect("non zero"), role);
//             }

//             // Check for the cached guild role ids
//             let cached_roles = cache
//                 .guild_roles(GuildId::new(1).expect("non zero"))
//                 .unwrap();
//             assert_eq!(cached_roles.len(), guild_1_role_ids.len());
//             assert!(guild_1_role_ids.iter().all(|id| cached_roles.contains(id)));

//             // Check for the cached role
//             assert!(guild_1_roles.into_iter().all(|role| cache
//                 .role(role.id)
//                 .expect("Role missing from cache")
//                 .resource()
//                 == &role))
//         }

//         // Bulk inserts
//         {
//             // The role ids for the guild with id 2
//             let guild_2_role_ids = (101..=110)
//                 .map(|n| RoleId::new(n).expect("non zero"))
//                 .collect::<Vec<_>>();
//             // Map the role ids to a test role
//             let guild_2_roles = guild_2_role_ids
//                 .iter()
//                 .copied()
//                 .map(test::role)
//                 .collect::<Vec<_>>();
//             // Cache all the roles using cache roles
//             cache.cache_roles(GuildId::new(2).expect("non zero"), guild_2_roles.clone());

//             // Check for the cached guild role ids
//             let cached_roles = cache
//                 .guild_roles(GuildId::new(2).expect("non zero"))
//                 .unwrap();
//             assert_eq!(cached_roles.len(), guild_2_role_ids.len());
//             assert!(guild_2_role_ids.iter().all(|id| cached_roles.contains(id)));

//             // Check for the cached role
//             assert!(guild_2_roles.into_iter().all(|role| cache
//                 .role(role.id)
//                 .expect("Role missing from cache")
//                 .resource()
//                 == &role))
//         }
//     }
// }
