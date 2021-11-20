use twilight_model::{id::UserId, user::User};

pub trait UserUtils {
    fn avatar_url(&self) -> UserAvatarBuilder;
}

impl UserUtils for User {
    fn avatar_url(&self) -> UserAvatarBuilder {
        UserAvatarBuilder::new(self.id, self.avatar.clone(), self.discriminator)
    }
}

pub struct UserAvatarBuilder {
    id: UserId,
    avatar: Option<String>,
    discriminator: u16,
    size: Option<u16>,
}

impl UserAvatarBuilder {
    /// Create a new user avatar builder.
    pub const fn new(id: UserId, avatar: Option<String>, discriminator: u16) -> Self {
        Self {
            id,
            avatar,
            discriminator,
            size: None,
        }
    }

    /// Build into an embed author.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use = "should be used as part of an embed builder"]
    pub fn build(self) -> String {
        if let Some(avatar) = &self.avatar {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}{}",
                self.id,
                avatar,
                if avatar.starts_with("a_") {
                    "gif"
                } else {
                    "png"
                },
                self.size_query()
            )
        } else {
            format!(
                "https://cdn.discordapp.com/embed/avatars/{}.png{}",
                self.discriminator % 5,
                self.size_query()
            )
        }
    }

    pub fn set_size(mut self, size: u16) -> Self {
        self.size = Some(if size >= 4096 {
            4096
        } else if size <= 16 {
            16
        } else {
            size - (size % 16)
        });

        self
    }

    fn size_query(&self) -> String {
        if let Some(size) = self.size {
            format!("?size={}", size)
        } else {
            "".into()
        }
    }
}
