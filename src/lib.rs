#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate err_derive;
pub mod backend;
pub mod stories;
pub mod users;
pub mod render;
use comrak::{Options, markdown_to_html};
use rmp_serde::encode;
use rocket::{FromForm, FromFormField};
use rocket_dyn_templates::{Template, context};
use std::path::Path;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, FromFormField, Copy)]
pub enum Owner {
    Group,
    User,
}

#[derive(Debug, Clone, Serialize, FromFormField, Deserialize, Copy, PartialEq, Eq)]
pub enum AccessLevel {
    Public,
    Group,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Ownership {
    pub owner_id: Uuid,
    pub owner: Owner,
    pub access: AccessLevel,
}

impl Ownership {
    pub fn public(user_id: Uuid) -> Self {
        Self {
            owner_id: user_id,
            owner: Owner::User,
            access: AccessLevel::Public,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryMeta {
    pub id: Uuid,
    pub description: Option<String>,
    pub owner: Ownership,
    /// ids for different characters
    pub characters: Vec<Uuid>,
    /// ids for different locations / settings within the story
    pub places: Vec<Uuid>,
    pub tags: Vec<String>,
    pub started: u64,
    pub last_editted: u64,
    /// link to other stories in either a timeline, or a universe etc
    pub links: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryFragment {
    pub meta: Uuid,
    pub name: String,
    pub content: Vec<u8>,
}