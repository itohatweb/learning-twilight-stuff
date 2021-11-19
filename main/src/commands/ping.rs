use twilight_model::application::{callback::InteractionResponse, interaction::ApplicationCommand};
use twilight_util::builder::CallbackDataBuilder;
use twilight_util::snowflake::Snowflake;

use crate::types::Context;

use super::ExecCommandError;

pub async fn run(context: &Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    let ping_time = chrono::Utc::now().timestamp_millis() - command.id.timestamp();

    let guild_count = context.cache.iter().guilds().count();

    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new()
                    .content(format!(
                        "PONG! Ping time: {}ms\nCached Guilds: {}",
                        ping_time, guild_count
                    ))
                    .build(),
            ),
        )
        .await
        .unwrap();

    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new().content("foo".into()).build(),
            ),
        )
        .await
        .unwrap();

    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new().content("bar".into()).build(),
            ),
        )
        .await
        .unwrap();

    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new().content("baz".into()).build(),
            ),
        )
        .await
        .unwrap();

    Ok(())
}
