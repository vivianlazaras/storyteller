use crate::ApiClient;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, form::Form, get, post, response::Redirect, routes};
use std::collections::HashMap;
use uuid::Uuid;
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

/*pub struct RelationBuilder<'l> {
    parent: String,
    category: String,
}

impl<'l> RelationBuilder<'l> {
    pub fn new(parent: Uuid, category: String) -> Self {
        let parent = parent.to_string();
        Self {
            parent,
            category
        }
    }
    pub fn get_params() -> HashMap<&'static str, &'l str> {
        let mut params = HashMap::new();
        params.insert("parent", self.parent.as_str());
        params.insert("category", self.category.as_str())
        params
    }
}*/

use rocket_dyn_templates::{Template, context};
#[get("/create?<id>&<parent>&<child>")]
async fn create_link_html(
    api: &State<ApiClient>,
    id: Uuid,
    parent: String,
    child: String,
) -> RawHtml<Template> {
    let mut params = HashMap::new();
    params.insert("category", child.as_str());
    let items: Option<Vec<RelatedEntity>> = api.get("/relations", Some(params)).await.unwrap();
    RawHtml(Template::render(
        "links/create",
        context! { title: "create link", category: child, parent_category: parent, parent: id, items },
    ))
}

#[post("/", data = "<rel>")]
async fn create_link(api: &State<ApiClient>, rel: Form<Relation>) -> Redirect {
    let relation = rel.into_inner();
    let redirect = format!("/{}/{}", relation.parent_category, relation.parent);
    let _: Relation = api.post("/relations/", "", None, relation).await.unwrap();
    Redirect::to(redirect)
}

pub fn get_routes() -> Vec<Route> {
    routes![create_link_html, create_link]
}
