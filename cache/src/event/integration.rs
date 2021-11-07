use crate::{config::ResourceType, GuildResource, InRedisCache, UpdateCache};
use twilight_model::{
    gateway::payload::incoming::{IntegrationCreate, IntegrationDelete, IntegrationUpdate},
    guild::GuildIntegration,
    id::{GuildId, IntegrationId},
};

impl InRedisCache {
    async fn cache_integration(&self, guild_id: GuildId, integration: GuildIntegration) {
        // self.guild_integrations
        //     .entry(guild_id)
        //     .or_default()
        //     .insert(integration.id);

        self.guild_integrations
            .insert(guild_id.get(), &integration.id.get())
            .await;

        self.integrations
            .insert(
                (guild_id.get(), integration.id.get()),
                GuildResource {
                    guild_id,
                    value: integration,
                },
            )
            .await;
    }

    async fn delete_integration(&self, guild_id: GuildId, integration_id: IntegrationId) {
        self.integrations
            .delete((guild_id.get(), integration_id.get()))
            .await;
        self.guild_integrations
            .remove(guild_id.get(), integration_id.get())
            .await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for IntegrationCreate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::INTEGRATION) {
            return;
        }

        if let Some(guild_id) = self.guild_id {
            cache
                .integrations
                .insert(
                    (guild_id.get(), self.id.get()),
                    GuildResource {
                        guild_id,
                        value: self.0.clone(),
                    },
                )
                .await;
        }
    }
}

#[async_trait::async_trait]
impl UpdateCache for IntegrationDelete {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::INTEGRATION) {
            return;
        }

        cache.delete_integration(self.guild_id, self.id).await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for IntegrationUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::INTEGRATION) {
            return;
        }

        if let Some(guild_id) = self.guild_id {
            cache.cache_integration(guild_id, self.0.clone()).await;
        }
    }
}
