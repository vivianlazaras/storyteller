use rocket_oidc::auth::AuthGuard;

pub(crate) type Guard = AuthGuard<UserGuard>;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
pub struct UserGuard {
    pub email: String,
    pub sub: String,
    pub picture: Option<String>,
    pub email_verified: Option<bool>,
}
