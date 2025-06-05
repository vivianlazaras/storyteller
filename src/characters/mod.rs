pub mod api;
pub(crate) mod frontend;
use api::*;
pub use frontend::get_routes;
use rocket::{routes, Route};

