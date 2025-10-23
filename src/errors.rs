use rocket::http::Status;
use rocket::{
    Request, get,
    response::{Redirect},
};
use rocket_dyn_templates::{Template, context};
use std::io::Cursor;

use reqwest::StatusCode;
use std::error::Error;
use std::fmt;
use rocket::response::{Responder, Result as RocketResult};

#[derive(Debug, Clone)]
pub enum ApiError {
    Forbidden(String),
    AccessDenied(String),
    NotFound(String),
    BadRequest(String),
    UnprocessableEntity(String),
    InternalServerError(String),
    Unavailable(String),
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> RocketResult<'static> {
        println!("LOG [ERROR] {:?}", self);
        let status = match self {
            ApiError::Forbidden(_) => Status::Forbidden,
            ApiError::AccessDenied(_) => Status::Unauthorized,
            ApiError::NotFound(_) => Status::NotFound,
            ApiError::BadRequest(_) => Status::BadRequest,
            ApiError::UnprocessableEntity(_) => Status::UnprocessableEntity,
            ApiError::InternalServerError(_) => Status::InternalServerError,
            ApiError::Unavailable(_) => Status::ServiceUnavailable,
        };

        // Return Err to trigger Rocket's catcher for the given status
        Err(status)
    }
}

macro_rules! server_error {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ApiError {
                fn from(err: $t) -> ApiError {
                    ApiError::InternalServerError(err.to_string())
                }
            }
        )*
    };
}

macro_rules! init_error {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ApiError {
                fn from(err: $t) -> ApiError {
                    ApiError::Unavailable(err.to_string())
                }
            }
        )*
    };
}

server_error!(
    std::io::Error,
    std::fmt::Error,
    anyhow::Error,
    url::ParseError,
    image::ImageError,
    nom_exif::Error,
    reqwest::Error
);

init_error!(jsonwebtoken::errors::Error, base64::DecodeError);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Forbidden(res) => write!(
                f,
                "Authentication successful but resource access denied {}",
                res
            ),
            ApiError::AccessDenied(res) => write!(f, "unauthorized access denied {}", res),
            ApiError::NotFound(res) => write!(f, "resource {} not found", res),
            ApiError::BadRequest(req) => write!(f, "bad request {}", req),
            ApiError::UnprocessableEntity(entity) => write!(f, "failed to process: {}", entity),
            ApiError::InternalServerError(err) => write!(f, "internal server error {}", err),
            ApiError::Unavailable(init) => write!(
                f,
                "server failed to initalize properly service unavailble {}",
                init
            ),
        }
    }
}

impl ApiError {
    pub fn from_status(status: StatusCode, message: String) -> Self {
        match status {
            StatusCode::NOT_FOUND => ApiError::NotFound(message),
            StatusCode::UNAUTHORIZED => ApiError::AccessDenied(message),
            _ => ApiError::InternalServerError(message),
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

pub type LazyError = ApiError;
