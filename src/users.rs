use rocket_dyn_templates::{Template, context};
use rocket_oidc::{CoreClaims, OIDCGuard};
use uuid::Uuid;

use rocket::response::{Redirect, content::RawHtml};
use rocket::{Route, get, routes, http::{Cookie, CookieJar}};

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
    email: String,
    sub: String,
    picture: Option<String>,
    email_verified: Option<bool>,
}

impl CoreClaims for UserGuard {
    fn subject(&self) -> &str {
        self.sub.as_str()
    }
}

type Guard = OIDCGuard<UserGuard>;

#[get("/account")]
async fn account() -> Redirect {
    Redirect::to("/auth/keycloak")
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

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named("access_token"));
    Redirect::to("/")
}
pub fn get_routes() -> Vec<Route> {
    routes![account, profile, logout]
}
