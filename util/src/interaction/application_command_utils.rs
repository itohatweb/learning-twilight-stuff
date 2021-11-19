use twilight_model::{application::interaction::ApplicationCommand, user::User};

pub trait ApplicationCommandUtils {
    fn get_user(&self) -> User;
}

impl ApplicationCommandUtils for ApplicationCommand {
    fn get_user(&self) -> User {
        if let Some(member) = &self.member {
            return member.user.clone().unwrap();
        }

        self.user.clone().unwrap()
    }
}
