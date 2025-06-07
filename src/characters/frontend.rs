use crate::{
    ApiClient,
    assets::images::ImageProcessor,
    model::Character,
};
use rocket::{
    Route, State,
    form::{Form, FromForm},
    fs::TempFile,
    get, post,
    response::{Redirect, content::RawHtml},
    routes,
};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::api::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRender {
    pub image: Option<String>,
    pub tags: Vec<String>,
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[post("/", data = "<form>")]
async fn create_character<'f>(
    api: &State<ApiClient>,
    processor: &State<ImageProcessor>,
    form: Form<CharacterBuilderForm<'f>>,
) -> Redirect {
    let form = form.into_inner();
    let character_builder = form.to_builder();
    let character = character_builder.build(&api, "").await.unwrap();

    if let Some(image_builder) = character.build_image(form, processor).await.unwrap() {
        image_builder.build(&api, "").await.unwrap();
        // If needed, you can associate the image_builder with character here.
        // E.g., character.image = Some(image_builder.id());
    }

    Redirect::to("/characters/")
}

#[get("/<id>")]
async fn get_character(id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let character: Character = api.get(&format!("/characters/{}", id), None).await.unwrap();
    let render = character.render(Some(String::from("/images/debe1a6f-5f7f-4cf4-84ef-e913efaa8dcd")), Vec::new());
    RawHtml(Template::render(
        "characters/character",
        context! { title: render.name.clone(), character: render },
    ))
}

#[get("/create")]
async fn create_character_html() -> RawHtml<Template> {
    RawHtml(Template::render(
        "characters/create",
        context! { title: "Create New Character" },
    ))
}

#[get("/")]
async fn list_characters(api: &State<ApiClient>) -> RawHtml<Template> {
    let characters: Option<Vec<Character>> = api.get("/characters", None).await.unwrap();
    RawHtml(Template::render(
        "characters/index",
        context! { title: "Characters", characters },
    ))
}

#[derive(FromForm, Debug)]
pub struct CharacterBuilderForm<'r> {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub imagetags: Option<Vec<String>>,
    pub image: Option<TempFile<'r>>,
}

impl<'r> CharacterBuilderForm<'r> {
    pub fn to_builder(&self) -> CharacterBuilder {
        CharacterBuilder {
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
        }
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_character_html,
        list_characters,
        get_character,
        create_character
    ]
}