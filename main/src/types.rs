use std::sync::Arc;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_http::Client as HttpClient;
use twilight_model::application::command::Command as ApplicationCommand;
use twilight_model::{application::command::CommandOption, id::ApplicationId};
use util::interaction::responder::InteractionResponder;

// pub type Acache = std::sync::Arc<InMemoryCache>;

pub type Context = Arc<InnerContext>;

pub struct InnerContext {
    pub http: HttpClient,
    pub inter: InteractionResponder,
    pub cache: InMemoryCache,
}

impl InnerContext {
    pub fn new(
        token: String,
        application_id: ApplicationId,
        cache_resource_types: ResourceType,
    ) -> Self {
        // HTTP is separate from the gateway, so create a new client.
        let http = HttpClient::new(token.clone());

        // TODO: Interaction HTTP handler
        let http_inter = HttpClient::new(token);
        http_inter.set_application_id(application_id);

        let inter = InteractionResponder::new(http_inter);

        let cache = InMemoryCache::builder()
            .resource_types(cache_resource_types)
            .build();

        Self { http, inter, cache }
    }
}

#[async_trait::async_trait]
pub trait Command {
    async fn exec(context: Context, options: Vec<CommandOption>) -> Result<(), String>;
    fn build() -> ApplicationCommand;
}
