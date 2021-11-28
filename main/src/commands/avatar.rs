use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder, ImageSource};
use twilight_model::application::{
    callback::InteractionResponse,
    command::{Command, CommandType},
    interaction::{application_command::CommandOptionValue, ApplicationCommand},
};
use twilight_util::builder::{
    command::{CommandBuilder, UserBuilder},
    CallbackDataBuilder,
};
use util::{interaction::ApplicationCommandUtils, url::user::UserUtils};

use crate::types::Context;

use super::ExecCommandError;

pub async fn run(context: &Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    let user_id = match command.data.options.get(0) {
        Some(option) => match option.value {
            CommandOptionValue::User(user) => Some(user),
            _ => None,
        },
        _ => None,
    };

    let user = match user_id {
        Some(user_id) => command
            .data
            .resolved
            .as_ref()
            .map(|r| r.users.iter().find(|u| u.id == user_id))
            .flatten()
            .map(|v| v.to_owned()),
        None => Some(command.get_user()),
    };

    let user = match user {
        Some(user) => user,
        None => {
            return Err(ExecCommandError::Message(
                "The user could not be found".into(),
            ))
        }
    };

    let image_source = ImageSource::url(user.avatar_url().set_size(4096).build()).unwrap();

    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new()
                    // .components([component])
                    .embeds(vec![EmbedBuilder::new()
                        .title("Avatar Link")
                        .url(user.avatar_url().set_size(1024).build())
                        .image(image_source)
                        .footer(
                            EmbedFooterBuilder::new(format!(
                                "by {}#{}",
                                command.get_user().name,
                                command.get_user().discriminator(),
                            ))
                            .icon_url(
                                ImageSource::url(command.get_user().avatar_url().build()).unwrap(),
                            ),
                        )
                        .color(240116)
                        .build()
                        .unwrap()])
                    .build(),
            ),
        )
        .await
        .unwrap();

    Ok(())
}

pub fn build() -> Command {
    CommandBuilder::new(
        "avatar".to_string(),
        "🖼️ Show the avatar of a user or yourself.".to_string(),
        CommandType::ChatInput,
    )
    .option(UserBuilder::new(
        "user".to_string(),
        "Provide @user to view their avatar.".to_string(),
    ))
    .build()
}
