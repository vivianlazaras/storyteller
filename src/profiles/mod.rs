use rocket_dyn_templates::{Template, context};
use rocket_oidc::{CoreClaims, OIDCGuard};
use uuid::Uuid;

use crate::ApiClient;
use rocket::response::{Redirect, content::RawHtml};
use rocket::{
    Route, get, post, form::Form, FromForm, State, routes,
    http::{Cookie, SameSite, CookieJar},
    
};

pub fn default_profile_url() -> String {
    String::from("/static/profile.jpg")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
pub struct UserGuard {
    pub email: String,
    pub sub: String,
    pub picture: Option<String>,
    pub email_verified: Option<bool>,
}

impl CoreClaims for UserGuard {
    fn subject(&self) -> &str {
        self.sub.as_str()
    }
}

pub(crate) type Guard = OIDCGuard<UserGuard>;

#[get("/account")]
async fn account() -> Redirect {
    Redirect::to("/profiles/login")
}

#[get("/login")]
async fn login_page() -> RawHtml<Template> {
    RawHtml(Template::render(
        "profiles/login",
        context!( title: "login", oidc_url: "/auth/keycloak" )
    ))
}

#[get("/profile")]
async fn profile(guard: Guard) -> RawHtml<Template> {
    let image_url = match &guard.claims.picture {
        Some(picture) => picture.clone(),
        None => default_profile_url(),
    };
    println!("guard: {:?}", guard);
    RawHtml(Template::render(
        "profile",
        context!( title: "profile", name: guard.claims.email, info: guard.userinfo, picture: image_url ),
    ))
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[post("/login", data = "<form>")]
async fn login(api: &State<ApiClient>, form: Form<LoginForm>, jar: &CookieJar<'_>) -> Redirect {
    let login = form.into_inner();
    let access_token: String = match api.post("/login", "", None, login).await {
        Ok(token) => {
            // login has succeeded server should've responded with a signed json web token
            token
        },
        Err(e) => {
            return Redirect::to("/profiles/login")
        },
    };
    jar.add(
        Cookie::build((
            "access_token",
            access_token,
        ))
        .secure(false)
        .http_only(true) // good practice
        .same_site(SameSite::Lax), // or SameSite::Strict, if you prefer
    );
    Redirect::to("/profiles/login")
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named("access_token"));
    Redirect::to("/")
}
pub fn get_routes() -> Vec<Route> {
    routes![account, profile, login_page, logout, login]
}
