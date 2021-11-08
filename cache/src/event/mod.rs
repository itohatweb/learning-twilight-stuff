use std::borrow::Cow;

use twilight_model::{
    id::GuildId,
    user::{CurrentUser, User},
};

use crate::InRedisCache;

mod channel;
mod emoji;
mod guild;
mod integration;
mod interaction;
mod member;
mod message;
mod presence;
mod reaction;
mod role;
mod stage_instance;
// mod sticker;
mod thread;
// mod voice_state;

impl InRedisCache {
    // TODO: cache
    // fn cache_current_user(&self, current_user: CurrentUser) {
    //     self.current_user
    //         .lock()
    //         .expect("current user poisoned")
    //         .replace(current_user);
    // }

    pub(crate) async fn cache_user(&self, user: Cow<'_, User>, guild_id: Option<GuildId>) {
        match self.users.get(user.id.get()).await {
            Some(u) if &u == user.as_ref() => {
                if let Some(guild_id) = guild_id {
                    self.user_guilds
                        .insert(guild_id.get(), &user.id.get())
                        .await;
                }

                return;
            }
            Some(_) | None => {}
        }
        let user = user.into_owned();
        let user_id = user.id;

        self.users.insert(user_id.get(), user).await;

        if let Some(guild_id) = guild_id {
            self.user_guilds.insert(user_id.get(), &guild_id.get());
        }
    }

    async fn unavailable_guild(&self, guild_id: GuildId) {
        self.unavailable_guilds
            .insert("unavailable_guilds".into(), &guild_id.get())
            .await;
        self.guilds.delete(guild_id.get());
    }
}
