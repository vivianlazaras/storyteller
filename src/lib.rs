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
use reqwest::Url;
use std::path::PathBuf;
use rocket_oidc::OIDCConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    backend: String,
    name: String,
    user: String,
    host: String,
    port: u16,
    passwordFile: Option<PathBuf>,
}

impl Default for DatabaseConfig {
    fn default() -> DatabaseConfig {
        DatabaseConfig {
            backend: "postgres".to_string(),
            host: "localhost".to_string(),
            user: "storyteller".to_string(),
            port: 5432,
            name: "storyteller".to_string(),
            passwordFile: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    port: u16,
    listen: String,
    url: String,
    ssl: bool,
    certFile: Option<PathBuf>,
    keyFile: Option<PathBuf>,
    oidc: OIDCConfig,
}

impl ServerConfig {
    pub fn endpoint(&self) -> &str {
        &self.url
    }
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            port: 8000,
            listen: "localhost".to_string(),
            url: "localhost".to_string(),
            ssl: false,
            certFile: None,
            keyFile: None,
            oidc: OIDCConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    server: ServerConfig,
    db: DatabaseConfig,
}

impl APIConfig {
    pub fn endpoint(&self) -> &str {
        &self.server.endpoint()
    }
}

impl Default for APIConfig {
    fn default() -> APIConfig {
        let mut server = ServerConfig::default();
        server.port = 8442;
        APIConfig {
            server,
            db: DatabaseConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    server: ServerConfig,
    api: APIConfig,
}

impl Config {
    pub fn port(&self) -> u16 {
        self.server.port
    }

    pub fn listen(&self) -> &str {
        &self.server.listen
    }

    pub fn api_endpoint(&self) -> &str {
        &self.api.endpoint()
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut server = ServerConfig::default();
        server.port = 8440;
        Config {
            server,
            api: APIConfig::default(),
        }
    }
}

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
}