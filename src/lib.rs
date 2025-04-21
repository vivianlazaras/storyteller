#[macro_use]
extern crate serde_derive;
pub mod loader;
pub mod projects;
pub mod resources;
pub mod stories;
pub mod users;
use comrak::{Options, markdown_to_html};
use rmp_serde::encode;
use rocket::{FromForm, FromFormField};
use rocket_dyn_templates::{Template, context};
use sled::Tree;
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
pub struct Record {
    pub name: String,
    pub description: Option<String>,
    pub owner: Ownership,
    pub tags: Vec<String>,
}
