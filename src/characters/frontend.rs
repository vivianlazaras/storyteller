use super::api::*;
use crate::assets::graphs::render_graph;
use crate::assets::images::{ImageBuilder, ImageData, ImageForm};
use crate::get_access_token;
use crate::{ApiClient, assets::images::ImageProcessor, auth::Guard, model::Character};
use anyhow::Result;
use rocket::{
    Route, State,
    form::{Form, FromForm},
    fs::TempFile,
    get,
    http::CookieJar,
    post,
    response::{Redirect, content::RawHtml},
    routes,
};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[post("/", data = "<form>")]
async fn create_character<'f>(
    guard: Guard,
    api: &State<ApiClient>,
    processor: &State<ImageProcessor>,
    form: Form<CharacterBuilderForm<'f>>,
) -> Redirect {
    let form = form.into_inner();
    let character_builder = form.into_builder(&processor).await.unwrap();
    let character = character_builder
        .build(&api, guard.access_token())
        .await
        .unwrap();

    /*if let Some(image_builder) = character.build_image(form, processor).await.unwrap() {
        image_builder.build(&api, &get_access_token(jar)).await.unwrap();
        // If needed, you can associate the image_builder with character here.
        // E.g., character.image = Some(image_builder.id());
    }*/

    Redirect::to("/characters/")
}

#[get("/<id>")]
async fn get_character(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let render: CharacterRender = api
        .get_protected(&format!("/characters/{}", id), guard.access_token(), None)
        .await
        .unwrap();
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
async fn list_characters(
    guard: Guard,
    api: &State<ApiClient>,
    jar: &rocket::http::CookieJar<'_>,
) -> RawHtml<Template> {
    println!("access_token: {}", guard.access_token());
    assert_eq!(&crate::get_access_token(jar), guard.access_token());
    let characters: Vec<CharacterRender> = match api
        .get_protected("/characters", &crate::get_access_token(jar), None)
        .await
        .unwrap()
    {
        Some(characters) => characters,
        None => Vec::new(),
    };
    println!("{:?}", serde_json::to_string(&characters));
    RawHtml(Template::render(
        "characters/index",
        context! { title: "Characters", characters },
    ))
}

#[get("/trees/<id>")]
async fn get_tree(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/characters/{}", id);
    let character: Character = api
        .get_protected(url, guard.access_token(), None)
        .await
        .unwrap();
    let (graph, index_map) = Character::family_tree(id, &api, guard.access_token())
        .await
        .unwrap();
    let svg = render_graph(graph);
    RawHtml(Template::render(
        "characters/tree",
        context! { title: character.name, tree: svg },
    ))
}

#[derive(Debug, Clone, FromForm)]
pub struct DeleteRequest {
    id: Uuid,
}

// this method could be the same for all entities
#[post("/delete", data = "<form>")]
async fn delete_entity(guard: Guard, form: Form<DeleteRequest>, api: &State<ApiClient>) {}

#[derive(FromForm, Debug)]
pub struct CharacterBuilderForm<'r> {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub imagetags: Option<Vec<String>>,
    pub images: Option<Vec<TempFile<'r>>>,
}

impl<'r> ImageForm<'r> for CharacterBuilderForm<'r> {
    fn images(&self) -> Option<&Vec<TempFile<'r>>> {
        self.images.as_ref()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tags(&self) -> &[String] {
        self.imagetags.as_deref().unwrap_or(&[])
    }

    fn category(&self) -> &str {
        "characters"
    }

    fn parent(&self) -> Option<Uuid> {
        None
    }
}

impl<'r> CharacterBuilderForm<'r> {
    pub async fn into_builder(self, processor: &ImageProcessor) -> Result<CharacterBuilder> {
        let thumbnail = self.into_image_builder(processor).await?;
        let tags = self.tags;
        let description = self.description;
        let name = self.name;

        Ok(CharacterBuilder {
            name,
            description,
            tags,
            thumbnail,
        })
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_character_html,
        list_characters,
        get_character,
        create_character,
        get_tree
    ]
}
