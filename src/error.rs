use rocket::{get, response::{Redirect, Responder}, Request};
use rocket_dyn_templates::{Template, context};
use rocket::http::Status;

// Custom error
pub enum FrontendError {
    BackendUnavailable,
    NotFound(String),
}

impl<'r> Responder<'r, 'static> for FrontendError {
    fn respond_to(self, _: &'r Request<'_>) -> Result<rocket::Response<'static>, Status> {
        let context = context! {
            message: "The backend service is currently unavailable. Please try again later."
        };
        Template::render("error", context).respond_to(&Request::default())
    }
}