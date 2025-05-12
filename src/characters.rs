use rocket::{
    Route, State, get,
    response::{Redirect, content::RawHtml},
    routes,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    identities: Vec<Uuid>,
    name: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    id: Uuid,
    name: String,
    image: Option<String>,
    description: Option<String>,
    age: u64,
}

pub fn get_routes() -> Vec<Route> {
    routes![]
}