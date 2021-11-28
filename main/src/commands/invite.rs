use twilight_model::{
    application::{
        callback::InteractionResponse,
        command::{Command, CommandType},
        interaction::ApplicationCommand,
    },
    channel::ReactionType,
    id::EmojiId,
};
use twilight_util::builder::{command::CommandBuilder, CallbackDataBuilder};
use util::builder::{ButtonBuilder, ComponentBuilder, SelectMenuBuilder, SelectMenuOptionBuilder};

use crate::types::Context;

use super::ExecCommandError;

pub async fn run(context: &Context, command: &ApplicationCommand) -> Result<(), ExecCommandError> {
    // let component = Component::ActionRow(ActionRow {
    //     components: Vec::from([Component::Button(Button {
    //         style: ButtonStyle::Primary,
    //         emoji: None,
    //         label: Some("test label".into()),
    //         custom_id: Some("test custom id".into()),
    //         url: None,
    //         disabled: false,
    //     })]),
    // });
    let new = ComponentBuilder::new()
        .button(
            ButtonBuilder::success("nicue".into())
                .emoji(ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(806599468504973332_u64).unwrap(),
                    name: None,
                })
                .label("fooo".into())
                .build()
                .unwrap(),
        )
        .button(
            ButtonBuilder::link("https://itoh.at/web".into())
                .emoji(ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(862112047558950912_u64).unwrap(),
                    name: Some("blurple_link".into()),
                })
                .label("linkedi link".into())
                .build()
                .unwrap(),
        )
        .button(
            ButtonBuilder::secondary("work".into())
                .emoji(ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(855134248603353098_u64).unwrap(),
                    name: None,
                })
                .disabled(true)
                .build()
                .unwrap(),
        )
        .select_menu(
            SelectMenuBuilder::new("testing menu".into())
                .add_options(vec![
                    SelectMenuOptionBuilder::new("lab-1".into(), "Lab 1".into())
                        .description("".into())
                        .build()
                        .unwrap(),
                    SelectMenuOptionBuilder::new("lab-2".into(), "Lab 2".into())
                        .emoji(ReactionType::Custom {
                            animated: true,
                            id: EmojiId::new(798140081713184798_u64).unwrap(),
                            name: Some("KEK".into()),
                        })
                        .build()
                        .unwrap(),
                ])
                .build(),
        )
        .button(
            ButtonBuilder::success("idk what this is".into())
                .label("baz".into())
                .build()
                .unwrap(),
        )
        // .select_menu(SelectMenu {
        //     custom_id: "test custom id 2asdfasdf".into(),
        //     disabled: false,
        //     placeholder: Some("adsfasdfaf".into()),
        //     min_values: None,
        //     max_values: None,
        //     options: vec![
        //         SelectMenuOption {
        //             label: "test option label".into(),
        //             value: "test option value".into(),
        //             description: Some("test description".into()),
        //             emoji: None,
        //             default: false,
        //         },
        //         SelectMenuOption {
        //             label: "test option label2".into(),
        //             value: "test option value2".into(),
        //             description: Some("test description2".into()),
        //             emoji: None,
        //             default: false,
        //         },
        //         SelectMenuOption {
        //             label: "test option label3".into(),
        //             value: "test option value3".into(),
        //             description: Some("test description3".into()),
        //             emoji: None,
        //             default: false,
        //         },
        //     ],
        // })
        .build();

    let _f = context
        .http
        .interaction_callback(
            command.id,
            &command.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackDataBuilder::new()
                    // .components([component])
                    .components(new)
                    .content("a content".into())
                    .build(),
                // CallbackData {
                //     allowed_mentions: None,
                //     components: Some(vec![component]),
                //     content: Some("a content".to_owned()),
                //     embeds: vec![],
                //     flags: None,
                //     tts: None,
                // },
            ),
        )
        .exec()
        .await?;

    Ok(())
}
