use anyhow::Result;

use log::error;
use twilight_http::Client;
use twilight_model::{
    application::{callback::InteractionResponse, interaction::ApplicationCommand},
    channel::message::MessageFlags,
    id::GuildId,
};
use twilight_util::builder::CallbackDataBuilder;

use crate::types::Context;

// Make every command a mod
pub mod avatar;
pub mod invite;
pub mod ping;

#[derive(Debug, thiserror::Error)]
pub enum ExecCommandError {
    #[error("The requested command `{0}` could not be found.")]
    CommandNotFound(String),
    #[error("Error occurred whilst sending a request: {0}")]
    HttpError(#[from] twilight_http::Error),

    /// Use this to automatically send an EPHEMERAL message to the executor.
    #[error("Something went wrong: {0}")]
    Message(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub async fn exec(context: Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    let result = match command.data.name.as_str() {
        "invite" => invite::run(&context, command).await,
        "ping" => ping::run(&context, command).await,
        "avatar" => avatar::run(&context, command).await,
        // _ => bail!("unknown command: {:?}", command),
        cn => Err(ExecCommandError::CommandNotFound(cn.into())),
    };

    if let Err(error) = result {
        use ExecCommandError::*;
        match &error {
            CommandNotFound(msg) => {
                context
                    .http
                    .interaction_callback(
                        command.id,
                        &command.token,
                        &InteractionResponse::ChannelMessageWithSource(
                            CallbackDataBuilder::new()
                                // TODO: get the subcommand names as well
                                .content(format!(
                                    "The requested command `{}` could not be found.",
                                    msg
                                ))
                                .flags(MessageFlags::EPHEMERAL)
                                .build(),
                        ),
                    )
                    .exec()
                    .await?;
            }
            Message(msg) => {
                context
                    .http
                    .interaction_callback(
                        command.id,
                        &command.token,
                        &InteractionResponse::ChannelMessageWithSource(
                            CallbackDataBuilder::new()
                                .content(msg.into())
                                .flags(MessageFlags::EPHEMERAL)
                                .build(),
                        ),
                    )
                    .exec()
                    .await?;
            }
            _ => {
                context
                    .http
                    .interaction_callback(
                        command.id,
                        &command.token,
                        &InteractionResponse::ChannelMessageWithSource(
                            CallbackDataBuilder::new()
                                .content("There was an error while executing this command.".into())
                                .flags(MessageFlags::EPHEMERAL)
                                .build(),
                        ),
                    )
                    .exec()
                    .await?;
            }
        }

        return Err(error);
    }

    // let callback = CallbackDataBuilder::new().content(reply).build();

    // ctx.http
    //     .interaction_callback(
    //         command.id,
    //         &command.token,
    //         &InteractionResponse::ChannelMessageWithSource(callback),
    //     )
    //     .exec()
    //     .await?;

    Ok(())
}
