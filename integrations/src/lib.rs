pub mod loader;
pub mod error;
pub mod data;
pub mod net;
pub mod daemon;
pub mod log;
use uuid::Uuid;

use serde_derive::*;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Category {
    Character,
    Story,
    Fragment,
    Timeline,
    Location,
    Link,
    Image,
    Video,
    Audio,
    Graphic
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[repr(C)]
pub enum Operation {
    Create,
    Read,
    Update,
    Delete,
    Index
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    category: Category,
    operations: Vec<Operation>,
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    id: Uuid,
    name: String,
    description: Option<String>,
    apikey: Option<String>,
    categories: Vec<Category>,

}