use crate::ApiClient;
use crate::model::*;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, delete, form::Form, get, post, put, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;
use super::api::*;
use rocket::fs::TempFile;

#[derive(Debug, FromForm)]
pub struct CreateFragmentForm<'r> {
    image: Option<TempFile<'r>>,
    content: String,
    name: String,
    parent: Option<Uuid>,
    category: Option<String>,
    tags: Vec<String>,
}

impl<'r> CreateFragmentForm<'r> {
    pub fn to_builder(&self) -> FragmentBuilder {
        let content = crate::normalize_newlines(&self.content);

        FragmentBuilder {
            parent: self.parent.clone(),
            category: self.category.as_deref(),
            name: self.name.as_str(),
            content: self.content.as_str(),
            tags: &self.tags
        }
    }
}

#[post("/", data = "<form>")]
async fn create_fragment<'r>(form: Form<CreateFragmentForm<'r>>, api: &State<ApiClient>) -> Redirect {
    let mut form = form.into_inner();
    let builder = form.to_builder();
    let newfragment: StoryFragment = builder.build(&api, "").await.unwrap();
    let redirect = if let Some(parent) = builder.parent {
        let category = match &builder.category {
            Some(category) => category,
            None => "stories",
        };
        format!("/{}/{}", category, parent)
    } else {
        format!("/fragments/{}", newfragment.id)
    };
    Redirect::to(redirect)
}

// id and category can be used to generate a redirect, and link automatically
#[get("/create?<id>&<category>")]
async fn create_fragment_html(
    api: &State<ApiClient>,
    id: Uuid,
    category: String,
) -> RawHtml<Template> {
    // id (the entity to link with)
    // category (the type of entity)

    let selected: Vec<String> = Vec::new();
    let options = api.get_top_tags(10, 0).await.unwrap();
    RawHtml(Template::render(
        "fragments/create",
        context! { title: "create new fragment", selected, options, parent: id, category },
    ))
}

#[get("/<id>")]
async fn get_fragment(id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/fragments/{}", id);
    let fragment: StoryFragment = api.get(&url, None).await.unwrap();
    RawHtml(Template::render(
        "fragments/fragment",
        context! { title: fragment.name.clone(), fragment },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_fragment, create_fragment_html, create_fragment]
}