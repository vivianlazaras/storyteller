use rocket::{
    Route, State, get,
    response::{Redirect, content::RawHtml},
    routes,
};

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    identities: Vec<Uuid>,
    name: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCharacter {
    name: String,
    image: Option<String>,
    description: Option<String>,
}

pub fn get_routes() -> Vec<Route> {
    routes![]
}