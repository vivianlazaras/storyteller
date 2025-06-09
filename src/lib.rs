#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate err_derive;
use time::{OffsetDateTime, format_description::well_known::Rfc3339, format_description};
use time::UtcOffset;

pub mod characters;
pub mod config;
mod model;
pub mod locations;
pub mod render;
pub mod errors;
pub mod stories;
pub mod profiles;
pub use config::Config;
pub mod fragments;
pub mod timelines;
pub mod assets;
pub mod links;
pub mod search;
pub mod notes;
pub mod themes;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use crate::errors::*;
use anyhow::Result;
use reqwest::Url;
use rocket::{FromForm, FromFormField};
use serde::Serialize;
use uuid::Uuid;

use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

#[derive(Deserialize)]
struct Jwk {
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
    r#use: Option<String>,
    kid: Option<String>,

    // Add other fields as needed
}

#[derive(Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

fn decoding_key_from_jwk(jwk_set: JwkSet) -> Result<DecodingKey> {

    let jwk = jwk_set
        .keys
        .first()
        .ok_or(ApiError::MissingJWTKey)?;

    if jwk.kty != "RSA" {
        return Err(ApiError::UnsupportedKeyType.into());
    }

    let modulus = URL_SAFE_NO_PAD.decode(&jwk.n)?;
    let exponent = URL_SAFE_NO_PAD.decode(&jwk.e)?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

    Ok(decoding_key)
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TagCount {
    value: String,
    count: i32,
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
            url: url.to_string(),
        })
    }
    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        route: &str,
        access_token: &str,
        params: Option<HashMap<&str, &str>>,
        data: T,
    ) -> Result<R> {
        println!("base url: {}", &self.url);
        let url = join_url(&self.url, route)?;

        let mut builder = self.client.post(&url);

        if let Some(params) = params {
            builder = builder.query(&params);
        }

        let response = builder
            .bearer_auth(access_token) // adds Authorization: Bearer <token>
            .json(&data)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            Ok(serde_json::from_str(&body)?)
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("API error: {} - {}", status, error_body))
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        route: &str,
        params: Option<HashMap<&'static str, &str>>,
    ) -> Result<T> {
        let url = join_url(&self.url, route)?;

        let mut builder = self.client.get(url);

        if let Some(params) = params {
            builder = builder.query(&params);
        }

        let response = builder.send().await?.error_for_status()?;
        let parsed = response.json::<T>().await?;
        Ok(parsed)
    }

    pub async fn get_top_tags(&self, limit: i32, min_count: i32) -> Result<Vec<TagCount>> {
        let mut params = HashMap::new();
        let limit_str = limit.to_string();
        let min_count_str = min_count.to_string();
        params.insert("limit", limit_str.as_str());
        params.insert("min_count", min_count_str.as_str());

        let options_opt: Option<Vec<TagCount>> =
            self.get("/analytics/populartags", Some(params)).await?;
        let options = match options_opt {
            Some(options) => options,
            None => Vec::new(),
        };
        Ok(options)
    }

    pub async fn delete(&self, baseurl: &str, id: Uuid) -> Result<()> {
        let baseurl = join_url(&self.url, baseurl).unwrap();
        let url = join_url(&baseurl, id.to_string().as_str())?;
        let response = self.client.delete(&url).send().await?;

        if response.status().is_success() {
            println!("Resource deleted successfully.");
        } else {
            println!("Failed to delete resource. Status: {}", response.status());
        }

        Ok(())
    }

    pub async fn get_jwt_pubkey(&self) -> Result<DecodingKey> {
        let keys: JwkSet = self.get("/pubkey", None).await?;
        Ok(decoding_key_from_jwk(keys)?)
    }
}

pub enum Category {
    Story,
    Character,
    Location,
    Timeline,
}

pub fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn epoch_to_human(epoch: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(epoch) {
        Ok(utc_dt) => {
            // Attempt to get local timezone offset at this datetime
            match UtcOffset::local_offset_at(utc_dt) {
                Ok(local_offset) => {
                    let local_dt = utc_dt.to_offset(local_offset);
                    let format = format_description::parse("[month]/[day]/[year] [hour]:[minute]").unwrap();
                    local_dt.format(&format).unwrap_or_else(|_| "Invalid format".to_string())
                }
                Err(_) => "Failed to get local timezone offset".to_string(),
            }
        }
        Err(_) => "Invalid epoch time".to_string(),
    }
}