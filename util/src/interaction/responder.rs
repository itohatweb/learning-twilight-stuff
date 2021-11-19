use dashmap::DashSet;
use twilight_model::{
    application::callback::InteractionResponse, channel::Message, id::InteractionId,
};

pub struct InteractionResponder {
    unreplied: DashSet<InteractionId>,
    // TODO: change this to the InteractionClient thingy: https://github.com/twilight-rs/twilight/pull/1275
    http: twilight_http::Client,
}

impl InteractionResponder {
    pub fn new(http: twilight_http::Client) -> Self {
        Self {
            // TODO: clear task?
            unreplied: DashSet::new(),
            http,
        }
    }

    /// Add a new interaction to the responder.
    /// Important for the first response.
    ///
    /// Example:
    /// ```
    /// let responder = InteractionResponder::new();
    ///
    /// match event {
    ///     Event::InteractionCreate(interaction) => {
    ///          responder.add_interaction(interaction.id());         
    ///     }
    /// }
    /// ```
    pub fn add_interaction(&self, interaction_id: InteractionId) {
        self.unreplied.insert(interaction_id);
    }

    /// Remove an interaction from the responder.
    /// Useful if you do not want to respond to an interaction at all and prevent memory leaks.
    /// Although this is not recommended.
    pub fn remove_interaction(&self, interaction_id: InteractionId) {
        self.unreplied.remove(&interaction_id);
    }

    /// Respond to an interaction, by ID and token.
    ///
    /// For variants of [`InteractionResponse`] that contain a [`CallbackData`],
    /// there is an [associated builder] in the [`twilight-util`] crate.
    ///
    /// [`CallbackData`]: twilight_model::application::callback::CallbackData
    /// [`twilight-util`]: https://docs.rs/twilight-util/latest/index.html
    /// [associated builder]: https://docs.rs/twilight-util/latest/builder/struct.CallbackDataBuilder.html
    pub async fn respond(
        &self,
        interaction_id: InteractionId,
        interaction_token: &str,
        response: &InteractionResponse,
        // TODO: better error
    ) -> Result<Option<twilight_http::Response<Message>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let res = if self.unreplied.contains(&interaction_id) {
            self.http
                .interaction_callback(interaction_id, interaction_token, response)
                .exec()
                .await?;

            self.remove_interaction(interaction_id);

            None
        } else {
            // TODO: remove unwrap
            let mut followup = self
                .http
                .create_followup_message(interaction_token)
                .unwrap();

            if let Some(callback_data) = match response {
                InteractionResponse::Pong => None,
                InteractionResponse::ChannelMessageWithSource(r) => Some(r),
                InteractionResponse::DeferredChannelMessageWithSource(r) => Some(r),
                InteractionResponse::DeferredUpdateMessage => None,
                InteractionResponse::UpdateMessage(r) => Some(r),
            } {
                if let Some(allowed_mentions) = &callback_data.allowed_mentions {
                    followup = followup.allowed_mentions(allowed_mentions);
                }

                if let Some(components) = &callback_data.components {
                    followup = followup.components(components)?;
                }

                if let Some(content) = &callback_data.content {
                    followup = followup.content(content);
                }

                followup = followup.embeds(&callback_data.embeds);

                if let Some(tts) = callback_data.tts {
                    followup = followup.tts(tts);
                }
            } // TODO return error if wrong response

            Some(followup.exec().await?)
        };

        self.unreplied.remove(&interaction_id);

        Ok(res)
    }
}
