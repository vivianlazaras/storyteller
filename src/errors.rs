use rocket::{get, response::{Redirect, Responder}, Request};
use rocket_dyn_templates::{Template, context};
use rocket::http::Status;

#[derive(Debug, Clone, Error)]
#[error(display = "api request failure: {}", _0)]
pub enum ApiError {
    #[error(display = "unsupported key type for JWT signing")]
    UnsupportedKeyType,
    #[error(display = "failed to get JWT Keys for API")]
    MissingJWTKey,
}

// Custom error
pub enum FrontendError {
    BackendUnavailable,
    NotFound(String),
}