use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder, ImageSource};
use twilight_model::application::{callback::InteractionResponse, interaction::ApplicationCommand};
use twilight_util::builder::CallbackDataBuilder;
use twilight_util::snowflake::Snowflake;
use util::interaction::ApplicationCommandUtils;

use crate::types::Context;

use super::ExecCommandError;

pub async fn run(context: &Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    let latency = chrono::Utc::now().timestamp_millis() - command.id.timestamp();
    // let guild_count = context.cache.iter().guilds().count();

    // context
    //     .inter
    //     .respond(
    //         command.id,
    //         &command.token,
    //         &InteractionResponse::ChannelMessageWithSource(
    //             CallbackDataBuilder::new()
    //                 .content(format!(
    //                     "PONG! Ping time: {}ms\nCached Guilds: {}",
    //                     ping_time, guild_count
    //                 ))
    //                 .build(),
    //         ),
    //     )
    //     .await
    //     .unwrap();

    let now = std::time::Instant::now();
    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new().content(format!("🏓")).build(),
            ),
        )
        .await
        .unwrap();
    let rtt = now.elapsed().as_millis();

    let gw_lat = context
        .cluster
        .shard(0)
        .map(|shard| {
            shard
                .info()
                .ok()
                .map(|info| info.latency().recent().back().map(|last| last.as_millis()))
        })
        .flatten()
        .flatten();

    let user = command.get_user();
    let icon_url = ImageSource::url(format!(
        "https://cdn.discordapp.com/avatars/{}/{}.png",
        user.id,
        user.avatar.unwrap(),
    ))
    .unwrap();

    fn get_stat_emoji(n: f64) -> String {
        if n <= 0.0 {
            // GRAY
            return "<:sf:911337663708168322>".into();
        }

        if n >= 5.0 {
            // RED
            "<:sr:911337663737520148>".into()
        } else if n >= 1.0 {
            // YELLOW
            "<:sy:911337663729135656>".into()
        } else {
            // GREEN
            "<:sg:911337663737511966>".into()
        }
    }

    context
        .http
        .update_interaction_original(&command.token)
        .unwrap()
        .content(None)
        .unwrap()
        .embeds(Some(&[EmbedBuilder::new()
            .title("**🏓 Pong!**")
            // .description(format!(
            //     "🕓 **Latency:** {} s  {}\n🔄 **RTT:** {} s  {}\n💗 **Gateway:** {} s  {}",
            //     latency as f64 / 1000.0,
            //     get_stat_emoji(latency as f64 / 1000.0),
            //     rtt as f64 / 1000.0,
            //     get_stat_emoji(rtt as f64 / 1000.0),
            //     if let Some(gw_lat) = gw_lat {
            //         gw_lat as f64 / 1000.0
            //     } else {
            //         -1.0
            //     },
            //     get_stat_emoji(if let Some(gw_lat) = gw_lat {
            //         gw_lat as f64 / 1000.0
            //     } else {
            //         -1.0
            //     }),
            // ))
            .description(format!(
                "{} **Latency:** {} s\n{} **RTT:** {} s\n{} **Gateway:** {} s",
                get_stat_emoji(latency as f64 / 1000.0),
                latency as f64 / 1000.0,
                get_stat_emoji(rtt as f64 / 1000.0),
                rtt as f64 / 1000.0,
                get_stat_emoji(if let Some(gw_lat) = gw_lat {
                    gw_lat as f64 / 1000.0
                } else {
                    -1.0
                }),
                if let Some(gw_lat) = gw_lat {
                    gw_lat as f64 / 1000.0
                } else {
                    -1.0
                },
            ))
            .footer(
                EmbedFooterBuilder::new(format!(
                    "by {}#{}",
                    command.get_user().name,
                    command.get_user().discriminator(),
                ))
                .icon_url(icon_url),
            )
            .color(51283)
            .build()
            .unwrap()]))
        .unwrap()
        .exec()
        .await?;

    context
        .inter
        .respond(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new().content(format!("blub")).build(),
            ),
        )
        .await
        .ok();

    Ok(())
}
