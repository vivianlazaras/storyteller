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
mod model;
pub mod config;
pub use config::Config;
pub mod search;
use std::collections::HashMap;

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
use reqwest::Url;
use std::path::PathBuf;
use rocket_oidc::OIDCConfig;

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

fn join_url(root: &str, route: &str) -> Result<String, url::ParseError> {
    let base = Url::parse(root)?;
    let joined = base.join(route)?;
    Ok(joined.into())
}

impl ApiClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = reqwest::Client::new();
        Ok(Self {
            client,
            url: url.to_string()
        })
    }
    pub async fn post<T: Serialize>(&self, route: &str, access_token: &str, data: T) -> Result<String> {
        println!("base url: {}", &self.url);
        let url = join_url(&self.url, route)?;
        let response = self.client
            .post(&url)
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

    pub async fn get<T: serde::de::DeserializeOwned>(&self, route: &str, params: Option<HashMap<&'static str, &str>>) -> Result<T> {
        let url = join_url(&self.url, route)?;
        
        let mut builder = self.client
            .get(url);

        if let Some(params) = params {
            builder = builder.query(&params);
        }
        
        let response = builder.send()
            .await?
            .error_for_status()?;
        let parsed = response.json::<T>().await?;
        Ok(parsed)
    }
}