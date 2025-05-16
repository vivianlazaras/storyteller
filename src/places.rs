use rocket::{
    Route, State, get,
    response::{Redirect, content::RawHtml},
    routes,
};

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    id: Uuid,
    name: String,
    description: Option<String>,
}