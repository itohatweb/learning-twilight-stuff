use twilight_model::application::{
    callback::InteractionResponse,
    command::{Command, CommandType},
    interaction::ApplicationCommand,
};
use twilight_util::builder::{command::CommandBuilder, CallbackDataBuilder};

use crate::types::Context;

use super::ExecCommandError;

pub async fn run(context: &Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new()
                    .content("test successful".into())
                    .build(),
            ),
        )
        .await?;

    Ok(())
}

pub fn build() -> Command {
    CommandBuilder::new(
        "test".to_string(),
        "🧪 Just a placeholder command used to test stuff.".to_string(),
        CommandType::ChatInput,
    )
    .build()
}
