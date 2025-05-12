use redis::{Client, Connection};
use crate::{StoryFragment, StoryMeta};
use crate::backend::Backend;
use redis::Commands;

pub struct RedisDB {
    client: Client,
}

#[derive(Debug, Error)]
#[error(display = "database error occured: {}", _0)]
pub enum RedisErr {
    #[error(display = "{}", _0)]
    RedisErr(#[error(source)] redis::RedisError),
    #[error(display = "{}", _0)]
    JsonErr(#[error(source)] serde_json::Error),
}

impl Backend for RedisDB {
    type Record = StoryFragment;
    type Err = RedisErr;
    fn read<I: ToString>(&self, id: I) -> Result<Self::Record, Self::Err> {
        let mut conn = self.client.get_connection()?;
        let val: String = conn.get(id.to_string())?;
        let record = serde_json::from_str(&val)?;
        Ok(record)
    }
}