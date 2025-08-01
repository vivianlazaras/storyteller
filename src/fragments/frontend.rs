use super::api::*;
use crate::ApiClient;
use crate::auth::Guard;
use crate::model::*;
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, form::Form, get, post, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;
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
            tags: &self.tags,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FragmentRender {
    pub id: Uuid,
    pub name: String,
    pub content: String,
    pub images: Option<Vec<Image>>,
    pub created: String,
    pub last_edited: String,
}

#[post("/", data = "<form>")]
async fn create_fragment<'r>(
    guard: Guard,
    form: Form<CreateFragmentForm<'r>>,
    api: &State<ApiClient>,
) -> Redirect {
    let form = form.into_inner();
    let builder = form.to_builder();
    let newfragment: StoryFragment = builder.build(&api, guard.access_token()).await.unwrap();
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
async fn get_fragment(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/fragments/{}", id);
    let fragment: StoryFragment = api
        .get_protected(&url, guard.access_token(), None)
        .await
        .unwrap();
    RawHtml(Template::render(
        "fragments/fragment",
        context! { title: fragment.name.clone(), fragment },
    ))
}

#[get("/")]
async fn list_fragments(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    let fragments: Vec<StoryFragment> = match api
        .get_protected("/fragments/", guard.access_token(), None)
        .await
        .unwrap()
    {
        Some(fragments) => fragments,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "fragments/index",
        context!( title: "fragments", fragments ),
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        get_fragment,
        create_fragment_html,
        create_fragment,
        list_fragments
    ]
}
