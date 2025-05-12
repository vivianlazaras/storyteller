#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "sled")]
pub mod sled;

#[cfg(feature = "redis")]
use crate::backend::redis::*;

use crate::render::Renderable;

#[cfg(feature = "redis")]
type Database = RedisDB;

#[cfg(not(feature = "redis"))]
type Database = SledDB;

use serde::{Serialize, Deserialize};

pub trait Backend {
    type Err: std::error::Error;
    type Record: Renderable;
    /// # Arguments:
    ///
    /// id: Some Identifier such as a Path, or UUID
    ///
    /// # Returns
    /// Some metadata object that can be serialized or deserialized
    fn read<I: ToString>(&self, id: I) -> Result<Self::Record, Self::Err>;
}