use crate::errors::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::DecodingKey;
use reqwest::StatusCode;
use reqwest::{Client, Method, Url};
use rocket::http::CookieJar;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use time::{OffsetDateTime, UtcOffset, format_description};
use uuid::Uuid;

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

fn decoding_key_from_jwk(jwk_set: JwkSet) -> Result<DecodingKey, ApiError> {
    let jwk = jwk_set
        .keys
        .first()
        .ok_or(ApiError::InternalServerError("Missing JWT Key".to_string()))?;

    if jwk.kty != "RSA" {
        return Err(ApiError::InternalServerError("Unsupported Key Type".to_string()).into());
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

pub type Map<'a> = HashMap<&'a str, &'a str>;

impl ApiClient {
    pub async fn new(url: &str) -> Result<Self, ApiError> {
        Ok(Self {
            client: Client::new(),
            url: Url::parse(url)?,
        })
    }

    pub async fn post<'a, T, R, P>(
        &self,
        route: P,
        access_token: &str,
        params: Option<Map<'a>>,
        data: T,
    ) -> Result<R, ApiError>
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

        let response = builder.bearer_auth(access_token).json(&data).send().await?;

        if response.status().is_success() {
            Ok(response.json::<R>().await?)
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(ApiError::from_status(status, error_body.to_string()))
        }
    }

    pub async fn put<'a, T, R, P>(
        &self,
        route: P,
        access_token: &str,
        data: T,
    ) -> Result<R, ApiError>
    where
        T: Serialize,
        R: DeserializeOwned,
        P: AsRef<str>,
    {
        let url = join_url(self.url.as_str(), route)?;
        let mut builder = self.client.put(url);

        let response = builder.bearer_auth(access_token).json(&data).send().await?;

        if response.status().is_success() {
            Ok(response.json::<R>().await?)
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(ApiError::from_status(status, error_body.to_string()))
        }
    }

    #[deprecated]
    pub async fn get<'a, T, P>(&self, route: P, params: Option<Map<'a>>) -> Result<T, ApiError>
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

    pub async fn get_top_tags(
        &self,
        limit: i32,
        min_count: i32,
    ) -> Result<Vec<TagCount>, ApiError> {
        let limit_str = limit.to_string();
        let min_count_str = min_count.to_string();
        let mut params: HashMap<&'static str, &str> = HashMap::new();
        params.insert("min_count", min_count_str.as_str());
        params.insert("limit", limit_str.as_str());
        Ok(self.get("/analytics/populartags", Some(params)).await?)
    }

    pub async fn delete<P>(&self, baseurl: P, id: Uuid) -> Result<(), ApiError>
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

    pub async fn get_jwt_pubkey(&self) -> Result<DecodingKey, ApiError> {
        let keys: JwkSet = self.get("/pubkey", None).await?;
        decoding_key_from_jwk(keys)
    }

    #[deprecated]
    pub async fn get_protected<'a, T: DeserializeOwned, P: AsRef<str>>(
        &self,
        route: P,
        access_token: &str,
        params: Option<Map<'a>>,
    ) -> Result<T, ApiError> {
        let url = join_url(&self.url, route.as_ref())?;
        let mut builder = self.client.get(url).bearer_auth(access_token);

        if let Some(p) = params {
            builder = builder.query(&p);
        }

        let response = builder.send().await?.error_for_status()?;
        let parsed = response.json::<T>().await?;
        Ok(parsed)
    }

    pub fn empty_request<'a>(&'a self) -> ApiRequest<'a> {
        ApiRequest {
            method: Method::GET,
            route: String::new(),
            client: &self.client,
            base_url: &self.url,
            params: None,
            access_token: None,
        }
    }

    pub fn request<'a>(&'a self, route: &'a str) -> ApiRequest<'a> {
        ApiRequest {
            method: Method::GET,
            route: route.to_string(),
            client: &self.client,
            base_url: &self.url,
            params: None,
            access_token: None,
        }
    }

    /*pub fn post<'a, T>(&'a self, route: &'a str, payload: T) -> ApiRequest<'a>
    where
        T: Serialize + Send + Sync + 'a,
    {
        self.request(route)
            .method(Method::POST)
            .data(payload)
    }*/
}

// -------------------- ApiRequest --------------------

#[derive(Debug, Clone)]
pub struct ApiRequest<'a> {
    method: Method,
    route: String,
    client: &'a Client,
    base_url: &'a Url,
    params: Option<HashMap<String, String>>,
    access_token: Option<&'a str>,
    //body: Option<Box<dyn serde::Serialize + Send + Sync + 'a>>,
}

impl<'a> ApiRequest<'a> {
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn route(mut self, route: &'a str) -> Self {
        self.route = route.to_string();
        self
    }

    pub fn params(mut self, params: Map<'a>) -> Self {
        let mut owned = HashMap::new();
        for (k, v) in params.into_iter() {
            owned.insert(k.to_string(), v.to_string());
        }
        self.params = Some(owned);
        self
    }

    pub fn set_param<S: AsRef<str>>(mut self, name: S, value: String) -> Self {
        if let Some(params) = &mut self.params {
            params.insert(name.as_ref().to_string(), value);
        } else {
            let mut params = HashMap::new();
            params.insert(name.as_ref().to_string(), value);
            self.params = Some(params);
        }
        self
    }

    /// adds to route
    pub fn append<S: AsRef<str>>(mut self, subpath: S) -> Result<Self, ApiError> {
        self.route = join_url(self.route, subpath.as_ref())?.to_string();
        Ok(self)
    }

    pub fn access_token(mut self, token: &'a str) -> Self {
        self.access_token = Some(token);
        self
    }

    /*pub fn data<T: Serialize + 'a + Send + Sync>(mut self, data: T) -> Self {
        self.body = Some(Box::new(data));
        self
    }*/

    pub async fn send<R: DeserializeOwned>(self) -> Result<R, ApiError> {
        let url = join_url(self.base_url.as_str(), self.route)?;
        let mut req = self.client.request(self.method.clone(), url);

        if let Some(params) = self.params {
            req = req.query(&params);
        }

        if let Some(token) = self.access_token {
            req = req.bearer_auth(token);
        }

        /*if let Some(body) = self.body {
            req = req.json(&body);
        }*/

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(response.json::<R>().await?)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::from_status(status, body.to_string()))
        }
    }

    /*pub async fn send_raw(self) -> Result<Response> {
        let url = join_url(self.base_url.as_str(), self.route)?;
        let mut req = self.client.request(self.method.clone(), url);

        if let Some(params) = self.params {
            req = req.query(&params);
        }

        if let Some(token) = self.access_token {
            req = req.bearer_auth(token);
        }

        /*if let Some(body) = self.body {
            req = req.json(&body);
        }*/

        Ok(req.send().await?)
    }*/
}

pub fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub(crate) fn epoch_to_human(epoch: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(epoch) {
        Ok(utc_dt) => match UtcOffset::local_offset_at(utc_dt) {
            Ok(local_offset) => {
                let local_dt = utc_dt.to_offset(local_offset);
                let format =
                    format_description::parse("[month]/[day]/[year] [hour]:[minute]").unwrap();
                local_dt
                    .format(&format)
                    .unwrap_or_else(|_| "Invalid format".to_string())
            }
            Err(_) => "Failed to get local timezone offset".to_string(),
        },
        Err(_) => "Invalid epoch time".to_string(),
    }
}

pub(crate) fn get_access_token(jar: &CookieJar<'_>) -> String {
    jar.get("access_token")
        .map(|c| c.value().to_string())
        .unwrap_or(String::from(""))
}

#[rocket::async_trait]
pub trait Builder: Serialize {
    fn route() -> &'static str;
    async fn build<R>(&self, api: &ApiClient) -> Result<R, ApiError>;
}

pub trait Entity {
    type Builder: Serialize;
    fn route() -> &'static str;
    fn id(&self) -> Uuid;
    fn into_builder(self) -> Self::Builder;
}
/*
I couldn't help but laugh a bit in my own head at "there are less positions available because of AI" AI is a tool like any other, it makes our lives easier, it automates tasks and makes work go by much faster, so why would there be less work if AI allows work to go by more efficiently. Well isn't is somewhat obvious, we don't do work to survive or to do something rewarding, we are all depressed and work is just "something you have to do" but why not use AI to make incredible things, why not re imagine how we exist in the world and do things because they challenge us? Of course generations of truama have left us feeling like we could never deserve to want anything, that what we have to deal with is simly too great a challenge, but I have to ask, is it?

Are we really afraid of doing the work? of dying? or are we afraid of success, afraid of what will happen if we do incredible things, yes we could do horrible things, but each time we learn, we get better. climate change was an issue of lack of knowledge and lack of caution, we have caution and knowledge now, so why not take the reigns of our own fates, why not build something incredible? what is holding us back?

Fear, we are afraid to try, but so what? if we never try we never truly live, we never truly become fully fledged individuals, its just a matter of keeping at it, and trying. Hell storyteller is heavily built by AI, but that's made it doable, that's made my life so much easier, AI is no different from a search engine, people are just using it as an excuse to keep doing the same thing day in and day out instead of imagining something new. maybe we are afraid to imagine? afraid to dream? but I ask why? why be afraid to dream, as long as we have food, water, shelter, touch, rest, purpose, safety what's the harm in dreaming?

I have a dream today, that one day we all can imagine a world without pain, without suffering, I have a dream that we can live our lives in peace, where creativity, and seeing the effect it has on the world, our art, is nothing more than a thought and it gets made real.

what if we could see and understand everything in the world around us, see the intricacy of how it all fits together, molds, changes, and grows, what if we could see, imagine, and truly understand the majesty of life, is that not worth facing down a little fear?
*/
