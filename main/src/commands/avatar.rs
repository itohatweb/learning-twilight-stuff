use twilight_embed_builder::{EmbedBuilder, ImageSource};
use twilight_model::application::{callback::InteractionResponse, interaction::ApplicationCommand};
use twilight_util::builder::CallbackDataBuilder;
use util::{interaction::ApplicationCommandUtils, url::user::UserUtils};

use crate::types::Context;

use super::ExecCommandError;

pub async fn run(context: &Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    let image_source =
        ImageSource::url(command.get_user().avatar_url().set_size(4096).build()).unwrap();
    let _f = context
        .http
        .interaction_callback(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new()
                    // .components([component])
                    .embeds(vec![EmbedBuilder::new()
                        .image(image_source)
                        .build()
                        .unwrap()])
                    .build(),
            ),
        )
        .exec()
        .await?;

    Ok(())
}
