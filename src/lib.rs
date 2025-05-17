#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate err_derive;

//pub mod backend;
pub mod stories;
pub mod users;
pub mod render;
pub mod characters;
pub mod places;
pub mod model;
use comrak::{Options, markdown_to_html};
use rmp_serde::encode;
use rocket::{FromForm, FromFormField};
use rocket_dyn_templates::{Template, context};
use std::path::Path;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use walkdir::WalkDir;
use anyhow::Result;
use serde::Serialize;
use rocket::http::CookieJar;

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

pub struct ApiClient {
    client: reqwest::Client,
    url: String,
}

impl ApiClient {
    pub async fn post<T: Serialize>(&self, cookies: &CookieJar<'_>, data: T) -> Result<String, >{
        let access_token = match cookies.get("access_token") {
            Some(cookie) => cookie.to_string(),
            None => return Err(anyhow::anyhow!("Access EError missing access token"))
        };
        
        let response = self.client
            .post(&self.url)
            .bearer_auth(access_token)  // adds Authorization: Bearer <token>
            .json(&data)
            .send()
            .await?;
        
        if response.status().is_success() {
            let body = response.text().await?;
            Ok(body)
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("API error: {} - {}", status, error_body))
        }
    }
}