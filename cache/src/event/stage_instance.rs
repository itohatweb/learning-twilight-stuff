use crate::{config::ResourceType, InRedisCache, UpdateCache};
use twilight_model::{
    channel::StageInstance,
    gateway::payload::incoming::{StageInstanceCreate, StageInstanceDelete, StageInstanceUpdate},
    id::{GuildId, StageId},
};

impl InRedisCache {
    pub(crate) async fn cache_stage_instances(
        &self,
        guild_id: GuildId,
        stage_instances: impl IntoIterator<Item = StageInstance>,
    ) {
        let mut stage_instances_to_cache = vec![];
        let mut guild_stage_instances = vec![];
        for stage_instance in stage_instances {
            guild_stage_instances.push(stage_instance.id.get());
            stage_instances_to_cache.push((stage_instance.id.get(), stage_instance));
        }

        self.guild_stage_instances
            .insert_multiple(guild_id.get(), guild_stage_instances)
            .await;
        self.stage_instances
            .insert_multiple(stage_instances_to_cache)
            .await;
    }

    async fn cache_stage_instance(&self, guild_id: GuildId, stage_instance: StageInstance) {
        self.guild_stage_instances
            .insert(guild_id.get(), &stage_instance.id.get())
            .await;

        self.stage_instances
            .insert(stage_instance.id.get(), stage_instance)
            .await;
    }

    async fn delete_stage_instance(&self, guild_id: GuildId, stage_id: StageId) {
        if true == self.stage_instances.delete(stage_id.get()).await {
            self.guild_stage_instances
                .remove(guild_id.get(), stage_id.get())
                .await;
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for StageInstanceCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::STAGE_INSTANCE) {
            return;
        }

        cache
            .cache_stage_instance(self.guild_id, self.0.clone())
            .await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for StageInstanceDelete {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::STAGE_INSTANCE) {
            return;
        }

        cache.delete_stage_instance(self.guild_id, self.id).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for StageInstanceUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::STAGE_INSTANCE) {
            return;
        }

        cache
            .cache_stage_instance(self.guild_id, self.0.clone())
            .await;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use twilight_model::{channel::stage_instance::PrivacyLevel, id::ChannelId};

//     #[test]
//     fn test_stage_channels() {
//         let cache = InRedisCache::new();

//         let stage_instance = StageInstance {
//             channel_id: ChannelId::new(1).expect("non zero"),
//             discoverable_disabled: true,
//             guild_id: GuildId::new(2).expect("non zero"),
//             id: StageId::new(3).expect("non zero"),
//             privacy_level: PrivacyLevel::GuildOnly,
//             topic: "topic".into(),
//         };

//         cache.update(&StageInstanceCreate(stage_instance.clone()));

//         {
//             let cached_instances = cache
//                 .guild_stage_instances(stage_instance.guild_id)
//                 .unwrap();
//             assert_eq!(1, cached_instances.len());
//         }

//         {
//             let cached_instance = cache.stage_instance(stage_instance.id).unwrap();
//             assert_eq!(stage_instance.topic, cached_instance.topic);
//         }

//         let new_stage_instance = StageInstance {
//             topic: "a new topic".into(),
//             ..stage_instance
//         };

//         cache.update(&StageInstanceUpdate(new_stage_instance.clone()));

//         {
//             let cached_instance = cache.stage_instance(stage_instance.id).unwrap();
//             assert_ne!(stage_instance.topic, cached_instance.topic);
//             assert_eq!(new_stage_instance.topic, "a new topic");
//         }

//         cache.update(&StageInstanceDelete(new_stage_instance));

//         {
//             let cached_instances = cache
//                 .guild_stage_instances(stage_instance.guild_id)
//                 .unwrap();
//             assert_eq!(0, cached_instances.len());
//         }

//         {
//             let cached_instance = cache.stage_instance(stage_instance.id);
//             assert!(cached_instance.is_none());
//         }
//     }
// }
