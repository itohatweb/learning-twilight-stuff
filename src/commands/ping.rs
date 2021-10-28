use anyhow::Result;
use twilight_model::application::{callback::InteractionResponse, interaction::ApplicationCommand};
use twilight_util::builder::CallbackDataBuilder;

use crate::types::TwHttpClient;

pub async fn execute(http: TwHttpClient, command: &ApplicationCommand) -> Result<()> {
    http.interaction_callback(
        command.id,
        &command.token,
        &InteractionResponse::ChannelMessageWithSource(
            CallbackDataBuilder::new()
                .content("foo bar baz".into())
                .build(),
        ),
    )
    .exec()
    .await?;

    Ok(())
}
