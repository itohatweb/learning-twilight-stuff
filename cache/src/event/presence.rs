use crate::{config::ResourceType, model::CachedPresence, InRedisCache, UpdateCache};
use twilight_model::{
    gateway::{payload::incoming::PresenceUpdate, presence::UserOrId},
    id::{GuildId, UserId},
};

const fn presence_user_id(user_or_id: &UserOrId) -> UserId {
    match user_or_id {
        UserOrId::User(u) => u.id,
        UserOrId::UserId { id } => *id,
    }
}

impl InRedisCache {
    pub(crate) async fn cache_presences(
        &self,
        guild_id: GuildId,
        presences: impl IntoIterator<Item = CachedPresence>,
    ) {
        let mut presences_to_cache = vec![];
        let mut guild_presences = vec![];

        for presence in presences {
            guild_presences.push(presence.user_id().get());
            presences_to_cache.push(((guild_id.get(), presence.user_id().get()), presence));
        }

        self.presences
            .insert_multiple(
                // presences
                //     .into_iter()
                //     .map(|p| ((guild_id.get(), p.user_id().get()), p))
                //     .collect(),
                presences_to_cache,
            )
            .await;

        self.guild_presences
            .insert_multiple(guild_id.get(), guild_presences)
            .await;
    }

    async fn cache_presence(&self, guild_id: GuildId, presence: CachedPresence) {
        self.presences
            .insert((guild_id.get(), presence.user_id().get()), presence)
            .await;
    }
}

#[async_trait::async_trait]
impl UpdateCache for PresenceUpdate {
    async fn update(&self, cache: &InRedisCache) {
        if !cache.wants(ResourceType::PRESENCE) {
            return;
        }

        let presence = CachedPresence {
            activities: self.activities.clone(),
            client_status: self.client_status.clone(),
            guild_id: self.guild_id,
            status: self.status,
            user_id: presence_user_id(&self.user),
        };

        cache.cache_presence(self.guild_id, presence).await;
    }
}
