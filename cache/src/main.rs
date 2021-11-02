use deadpool_redis::{
    redis::{cmd, AsyncCommands, RedisError},
    Config, Connection, Pool, Runtime,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[tokio::main]
async fn main() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Human {
        age: u32,
        name: String,
    }

    // let mut buf = Vec::new();
    let val = Human {
        age: 42,
        name: "John".into(),
    };

    let c: Cache<String, Human> = Cache::new("redis://127.0.0.1", "humans".into());

    c.set("I".into(), val).await.unwrap();

    let g = c.get("I".into()).await.unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Was not able to get redids db pool")]
    FailedToGetPool,
    #[error("Redis command error")]
    RedisError,
}

impl From<deadpool_redis::PoolError> for CacheError {
    fn from(_: deadpool_redis::PoolError) -> Self {
        Self::FailedToGetPool
    }
}
impl From<RedisError> for CacheError {
    fn from(_: RedisError) -> Self {
        Self::RedisError
    }
}

struct Cache<K, V>
where
    K: deadpool_redis::redis::ToRedisArgs + std::marker::Sync + std::marker::Send,
{
    name: String,
    pool: Pool,
    key_type: std::marker::PhantomData<K>,
    value_type: std::marker::PhantomData<V>,
}

impl<K, V> Cache<K, V>
where
    K: deadpool_redis::redis::ToRedisArgs + std::marker::Sync + std::marker::Send,
    // V: 'static + Serialize + Deserialize<'static>,
{
    pub fn new(connection_str: &str, map_name: String) -> Cache<K, V> {
        let cfg = Config::from_url(connection_str);
        let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

        Self {
            name: map_name,
            pool,
            key_type: std::marker::PhantomData,
            value_type: std::marker::PhantomData,
        }
    }

    pub async fn set<T>(&self, key: K, value: T) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let mut con = self.get_con().await?;

        let pack = rmp_serde::to_vec(&value).unwrap();

        con.hset(self.name.clone(), key, pack).await?;

        Ok(())
    }

    pub async fn get(&self, key: K) -> Result<V, CacheError>
    where
        V: DeserializeOwned,
    {
        let mut con = self.get_con().await?;

        let value: Vec<u8> = cmd("HGET")
            .arg(self.name.clone())
            .arg(key)
            .query_async(&mut con)
            .await?;

        let dec = rmp_serde::from_read(&*value).unwrap();

        Ok(dec)
    }

    pub async fn size(&self) -> Result<usize, CacheError> {
        let mut con = self.get_con().await?;

        let length: usize = con.hlen(self.name.clone()).await?;

        Ok(length)
    }

    async fn get_con(&self) -> Result<Connection, CacheError> {
        Ok(self.pool.get().await?)
    }
}
