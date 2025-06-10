use time::{OffsetDateTime, UtcOffset, format_description};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use reqwest::{Client, Url};
use std::collections::HashMap;
use uuid::Uuid;
use jsonwebtoken::DecodingKey;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use anyhow::Result;
use crate::errors::*;
use rocket::http::CookieJar;

#[derive(Deserialize)]
struct Jwk {
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
    r#use: Option<String>,
    kid: Option<String>,
}

#[derive(Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

fn decoding_key_from_jwk(jwk_set: JwkSet) -> Result<DecodingKey> {
    let jwk = jwk_set.keys.first().ok_or(ApiError::MissingJWTKey)?;

    if jwk.kty != "RSA" {
        return Err(ApiError::UnsupportedKeyType.into());
    }

    let _modulus = URL_SAFE_NO_PAD.decode(&jwk.n)?;
    let _exponent = URL_SAFE_NO_PAD.decode(&jwk.e)?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
    Ok(decoding_key)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TagCount {
    value: String,
    count: i32,
}

pub struct ApiClient {
    client: Client,
    url: Url,
}

fn join_url<B: AsRef<str>, P: AsRef<str>>(base: B, path: P) -> Result<Url, url::ParseError> {
    Url::parse(base.as_ref())?.join(path.as_ref())
}

pub type Map<'a> = HashMap<&'static str, &'a str>;

impl ApiClient {
    pub async fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            url: Url::parse(url)?,
        })
    }

    pub async fn post<'a, T, R, P>(&self, route: P, access_token: &str, params: Option<Map<'a>>, data: T) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
        P: AsRef<str>,
    {
        let url = join_url(self.url.as_str(), route)?;
        let mut builder = self.client.post(url);

        if let Some(params) = params {
            builder = builder.query(&params);
        }

        let response = builder
            .bearer_auth(access_token)
            .json(&data)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json::<R>().await?)
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("API error: {} - {}", status, error_body))
        }
    }

    pub async fn get<'a, T, P>(&self, route: P, params: Option<Map<'a>>) -> Result<T>
    where
        T: DeserializeOwned,
        P: AsRef<str>,
    {
        let url = join_url(self.url.as_str(), route)?;
        let mut builder = self.client.get(url);

        if let Some(params) = params {
            builder = builder.query(&params);
        }

        Ok(builder.send().await?.error_for_status()?.json().await?)
    }

    pub async fn get_top_tags(&self, limit: i32, min_count: i32) -> Result<Vec<TagCount>> {
        let limit_str = limit.to_string();
        let min_count_str = min_count.to_string();
        let mut params: HashMap<&'static str, &str> = HashMap::new();
        params.insert("min_count", min_count_str.as_str());
        params.insert("limit", limit_str.as_str());
        Ok(self.get("/analytics/populartags", Some(params)).await?)
    }

    pub async fn delete<P>(&self, baseurl: P, id: Uuid) -> Result<()>
    where
        P: AsRef<str>,
    {
        let url = join_url(self.url.as_str(), baseurl)?;
        let url = join_url(url.as_str(), &id.to_string())?;

        let response = self.client.delete(url).send().await?;

        if response.status().is_success() {
            println!("Resource deleted successfully.");
        } else {
            println!("Failed to delete resource. Status: {}", response.status());
        }

        Ok(())
    }

    pub async fn get_jwt_pubkey(&self) -> Result<DecodingKey> {
        let keys: JwkSet = self.get("/pubkey", None).await?;
        decoding_key_from_jwk(keys)
    }

    pub async fn get_protected<'a, T: DeserializeOwned, P: AsRef<str>>(
        &self,
        route: P,
        access_token: &str,
        params: Option<Map<'a>>,
    ) -> Result<T> {
        let url = join_url(&self.url, route.as_ref())?;
        let mut builder = self.client.get(url).bearer_auth(access_token);

        if let Some(p) = params {
            builder = builder.query(&p);
        }

        let response = builder.send().await?.error_for_status()?;
        let parsed = response.json::<T>().await?;
        Ok(parsed)
    }
}

pub fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub(crate) fn epoch_to_human(epoch: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(epoch) {
        Ok(utc_dt) => match UtcOffset::local_offset_at(utc_dt) {
            Ok(local_offset) => {
                let local_dt = utc_dt.to_offset(local_offset);
                let format = format_description::parse("[month]/[day]/[year] [hour]:[minute]").unwrap();
                local_dt.format(&format).unwrap_or_else(|_| "Invalid format".to_string())
            }
            Err(_) => "Failed to get local timezone offset".to_string(),
        },
        Err(_) => "Invalid epoch time".to_string(),
    }
}

pub(crate) fn get_access_token(jar: &CookieJar<'_>) -> String {
    jar.get("access_token").map(|c| c.value().to_string()).unwrap_or(String::from(""))
}