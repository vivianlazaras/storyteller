use crate::ApiClient;
use crate::model::*;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, delete, form::Form, get, post, put, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct CreateFragment {
    parent: Option<Uuid>,
    category: Option<String>,
    name: String,
    content: String,
    #[field(name = "tags")]
    pub tags: Vec<String>,
}

#[post("/", data = "<fragment>")]
async fn create_fragment(fragment: Form<CreateFragment>, api: &State<ApiClient>) -> Redirect {
    let fragment = fragment.into_inner();

    let newfragment: StoryFragment = api.post("/fragments/", "", &fragment).await.unwrap();
    let redirect = if let Some(parent) = fragment.parent {
        let category = match &fragment.category {
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
