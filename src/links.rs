use rocket::{routes, Route, State, get, response::Redirect, post, form::Form, FromForm};
use rocket::response::content::{RawHtml};
use uuid::Uuid;
use crate::ApiClient;
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct Relation {
    pub parent: Uuid,
    pub child: Uuid,
    pub parent_category: String,
    pub child_category: String,
    pub description: Option<String>,
}

use rocket_dyn_templates::{context, Template};
#[get("/create?<id>&<parent>&<child>")]
async fn create_link_html(api: &State<ApiClient>, id: Uuid, parent: String, child: String) -> RawHtml<Template> {
    let mut params = HashMap::new();
    params.insert("category", child.as_str());
    let items: Option<Vec<RelatedEntity>> = api.get("/relations", Some(params)).await.unwrap();
    RawHtml(
        Template::render("links/create", context! { title: "create link", category: child, parent_category: parent, parent: id, items })
    )
}

#[post("/", data = "<rel>")]
async fn create_link(api: &State<ApiClient>, rel: Form<Relation>) -> Redirect {
    let relation = rel.into_inner();
    let redirect = format!("/{}/{}", relation.parent_category, relation.parent);
    let _: Relation = api.post("/relations/", "", relation).await.unwrap();
    Redirect::to(redirect)
}

pub fn get_routes() -> Vec<Route> {
    routes![create_link_html, create_link]
}