use rocket::{
    Route, State, get,
    response::{Redirect, content::RawHtml},
    routes,
};
use crate::ApiClient;
use rocket::{post};

use rocket_dyn_templates::{Template, context};
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

#[get("/<id>")]
async fn get_character(id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    unimplemented!();
}

#[get("/create")]
async fn create_character_html() -> RawHtml<Template> {
    RawHtml(
        Template::render("characters/create", context!{ title: "create new character" })
    )
}

#[post("/")]
async fn create_character() {
    unimplemented!();
}

pub fn get_routes() -> Vec<Route> {
    routes![create_character_html, get_character, create_character]
}