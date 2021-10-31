use parking_lot::RwLock;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;

pub type TwHttpClient = Arc<HttpClient>;

pub type RCache = std::sync::Arc<RwLock<InMemoryCache>>;
