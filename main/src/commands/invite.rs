use anyhow::Result;
use twilight_model::{
    application::{
        callback::InteractionResponse,
        component::{
            button::ButtonStyle, select_menu::SelectMenuOption, ActionRow, Button, Component,
            SelectMenu,
        },
        interaction::ApplicationCommand,
    },
    channel::ReactionType,
    id::EmojiId,
};
use twilight_util::builder::CallbackDataBuilder;
use util::builder::ButtonBuilder;

use crate::types::TwHttpClient;
#[derive(Clone, Debug)]
#[must_use = "builders have no effect if unused"]
struct ComponentBuilder(Vec<Component>);

impl ComponentBuilder {
    /// Create a new builder to construct a Vec<[`Component`]>.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Consume the builder, returning a Vec<[`Component`]>.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use = "builders have no effect if unused"]
    pub fn build(self) -> Vec<Component> {
        self.0
    }

    pub fn button(mut self, button: Button) -> Self {
        // Get the last ActionRow
        let component = self
            .0
            .iter_mut()
            .rev()
            .find(|component| matches!(component, Component::ActionRow(_)));

        match component {
            // Check if the ActionRow still has place and then add the button
            Some(Component::ActionRow(action_row)) if action_row.components.len() < 5 => {
                action_row.components.push(Component::Button(button));
                self
            }
            // No ActionRow found or its full so create a new one
            _ => self.action_row(vec![Component::Button(button)]),
        }
    }

    pub fn select_menu(self, select_menu: SelectMenu) -> Self {
        self.action_row(vec![Component::SelectMenu(select_menu)])
    }

    pub fn action_row(mut self, components: Vec<Component>) -> Self {
        if self.is_full() {
            return self;
        }

        self.0.push(Component::ActionRow(ActionRow { components }));

        self
    }

    fn is_full(&self) -> bool {
        self.0.len() == 5
    }
}

pub async fn execute(http: TwHttpClient, command: &ApplicationCommand) -> Result<()> {
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
            ButtonBuilder::new(ButtonStyle::Success, "nicue".into())
                .emoji(ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(806599468504973332_u64).unwrap(),
                    name: None,
                })
                .label("fooo".into())
                .build(),
        )
        .button(
            ButtonBuilder::new(ButtonStyle::Link, "https://itoh.at/web".into())
                .emoji(ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(862112047558950912_u64).unwrap(),
                    name: Some("blurple_link".into()),
                })
                .label("linkedi link".into())
                .build(),
        )
        .button(
            ButtonBuilder::new(ButtonStyle::Secondary, "work".into())
                .emoji(ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(855134248603353098_u64).unwrap(),
                    name: None,
                })
                .disabled(true)
                .build(),
        )
        .button(Button {
            style: ButtonStyle::Primary,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id1".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id2".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id3".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id34".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id53".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id36".into()),
            url: None,
            disabled: false,
        })
        .button(Button {
            style: ButtonStyle::Danger,
            emoji: None,
            label: Some("oh yea it works".into()),
            custom_id: Some("test custom id37".into()),
            url: None,
            disabled: false,
        })
        .select_menu(SelectMenu {
            custom_id: "test custom id 2asdfasdf".into(),
            disabled: false,
            placeholder: Some("adsfasdfaf".into()),
            min_values: None,
            max_values: None,
            options: vec![
                SelectMenuOption {
                    label: "test option label".into(),
                    value: "test option value".into(),
                    description: Some("test description".into()),
                    emoji: None,
                    default: false,
                },
                SelectMenuOption {
                    label: "test option label2".into(),
                    value: "test option value2".into(),
                    description: Some("test description2".into()),
                    emoji: None,
                    default: false,
                },
                SelectMenuOption {
                    label: "test option label3".into(),
                    value: "test option value3".into(),
                    description: Some("test description3".into()),
                    emoji: None,
                    default: false,
                },
            ],
        })
        .build();

    let _f = http
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
