use crate::ApiClient;
use rocket::{FromForm, Route, State, form::Form, post, response::Redirect, routes};
use std::collections::HashMap;

use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: i64,
    completed: Option<i64>,
    deadline: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct CreateNote {
    name: String,
    description: Option<String>,
    deadline: Option<i64>,
}

#[post("/?<parent>&<category>", data = "<noteform>")]
async fn create_note(
    parent: Uuid,
    category: String,
    noteform: Form<CreateNote>,
    api: &State<ApiClient>,
) -> Redirect {
    let note = noteform.into_inner();
    let mut params = HashMap::new();
    let parent_str = parent.to_string();
    params.insert("entity", parent_str.as_str());
    params.insert("category", category.as_str());
    let _: Note = api.post("/notes/", "", Some(params), note).await.unwrap();
    let redirect = format!("/{}/{}", category, parent);
    Redirect::to(redirect)
}

pub fn get_routes() -> Vec<Route> {
    routes![create_note]
}
