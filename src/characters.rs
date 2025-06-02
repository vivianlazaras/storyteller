use crate::ApiClient;
use crate::model::Character;
use rocket::post;
use rocket::{
    FromForm, Route, State,
    form::Form,
    get,
    response::{Redirect, content::RawHtml},
    routes,
};

use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    identities: Vec<Uuid>,
    name: String,
    description: String,
}

#[get("/<id>")]
async fn get_character(id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/characters/{}", id);
    let character: Character = api.get(&url, None).await.unwrap();
    RawHtml(Template::render(
        "characters/character",
        context! { title: character.name.clone(), character },
    ))
}

#[get("/create")]
async fn create_character_html() -> RawHtml<Template> {
    RawHtml(Template::render(
        "characters/create",
        context! { title: "create new character" },
    ))
}

#[get("/")]
async fn list_characters(api: &State<ApiClient>) -> RawHtml<Template> {
    let characters: Option<Vec<Character>> = api.get("/characters", None).await.unwrap();
    RawHtml(Template::render(
        "characters/index",
        context! { title: "characters", characters },
    ))
}

#[derive(FromForm, Serialize, Deserialize, Debug, Clone)]
pub struct CreateCharacter {
    name: String,
    description: String,
}

#[post("/", data = "<form>")]
async fn create_character(api: &State<ApiClient>, form: Form<CreateCharacter>) -> Redirect {
    let character = form.into_inner();
    let _: Character = api.post("/characters/", "", character).await.unwrap();
    Redirect::to("/characters/")
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_character_html,
        list_characters,
        get_character,
        create_character
    ]
}
