use rocket::http::Status;
use rocket::{
    Request, get,
    response::{Redirect, Responder},
};
use rocket_dyn_templates::{Template, context};

use std::error::Error;
use std::fmt;

// Formerly #[derive(Debug, Clone, Error)]
#[derive(Debug, Clone)]
pub enum ApiError {
    UnsupportedKeyType,
    MissingJWTKey,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::UnsupportedKeyType => write!(f, "unsupported key type for JWT signing"),
            ApiError::MissingJWTKey => write!(f, "failed to get JWT Keys for API"),
        }
    }
}

impl Error for ApiError {}

// Custom error
#[derive(Debug)]
pub enum FrontendError {
    BackendUnavailable,
    NotFound(String),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrontendError::BackendUnavailable => write!(f, "backend unavailable"),
            FrontendError::NotFound(msg) => write!(f, "not found: {}", msg),
        }
    }
}

impl Error for FrontendError {}
