use rocket_dyn_templates::{Template, context};
use rocket_oidc::{CoreClaims, OIDCGuard};
use uuid::Uuid;

use crate::ApiClient;
use crate::auth::*;
use bcrypt::BcryptError;
use bcrypt::DEFAULT_COST;
use bcrypt::hash;
use rocket::response::{Redirect, content::RawHtml};
use rocket::{
    FromForm, Route, State,
    form::Form,
    get,
    http::{Cookie, CookieJar, SameSite},
    post, routes,
};
use rocket_oidc::auth::AuthGuard;

pub fn hash_password(plain: &str) -> Result<String, bcrypt::BcryptError> {
    // Hash the password using bcrypt with the default cost (12)
    hash(plain, DEFAULT_COST)
}

pub fn default_profile_url() -> String {
    String::from("/static/profile.jpg")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    given_name: String,
    family_name: String,
}

impl CoreClaims for UserGuard {
    fn subject(&self) -> &str {
        self.sub.as_str()
    }
}

#[get("/account")]
async fn account() -> Redirect {
    Redirect::to("/profiles/login")
}

#[get("/login?<redirect>")]
async fn login_page(redirect: Option<String>) -> RawHtml<Template> {
    RawHtml(Template::render(
        "profiles/login",
        context!( title: "login", redirect, oidc_url: "/auth/keycloak" ),
    ))
}

#[get("/profile")]
async fn profile(guard: Guard) -> RawHtml<Template> {
    let image_url = match &guard.claims.picture {
        Some(picture) => picture.clone(),
        None => default_profile_url(),
    };

    let info = UserInfo {
        given_name: "vivian".to_string(),
        family_name: "lazaras".to_string(),
    };
    println!("guard: {:?}", guard);
    RawHtml(Template::render(
        "profiles/profile",
        context!( title: "profile", name: guard.claims.email, info, picture: image_url ),
    ))
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
pub struct LoginForm {
    redirect: Option<String>,
    email: String,
    password: String,
}

impl LoginForm {
    pub fn build(self) -> Result<LoginBuilder, BcryptError> {
        LoginBuilder::new(self.email, self.password)
    }
}
/// A login builder is used instead of just login form as login builder
/// hashed the password before sending it to the API server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginBuilder {
    email: String,
    password: String,
}

impl LoginBuilder {
    pub fn new(email: String, password: String) -> Result<Self, BcryptError> {
        let hashed_password = hash_password(&password)?;
        Ok(Self {
            email,
            password: hashed_password,
        })
    }
}

#[post("/login", data = "<form>")]
async fn login(api: &State<ApiClient>, form: Form<LoginForm>, jar: &CookieJar<'_>) -> Redirect {
    let login = form.into_inner();
    let redirect = login.redirect.clone();
    let access_token: String = match api.post("/login", "", None, login).await {
        Ok(token) => {
            // login has succeeded server should've responded with a signed json web token
            token
        }
        Err(e) => return Redirect::to("/profiles/login"),
    };
    jar.add(
        Cookie::build(("access_token", access_token))
            .secure(false)
            .http_only(true) // good practice
            .same_site(SameSite::Lax), // or SameSite::Strict, if you prefer
    );
    match redirect {
        Some(redirect) => Redirect::to(redirect),
        None => Redirect::to("/profiles/profile"),
    }
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named("access_token"));
    Redirect::to("/")
}
pub fn get_routes() -> Vec<Route> {
    routes![account, profile, login_page, logout, login]
}
