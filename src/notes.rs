use crate::ApiClient;
use crate::auth::Guard;
use rocket::get;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, form::Form, post, response::Redirect, routes};
use rocket_dyn_templates::{Template, context};
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
pub struct NoteForm {
    name: String,
    description: Option<String>,
    deadline: Option<i64>,
    parent: Uuid,
    category: String,
}

#[post("/", data = "<noteform>")]
async fn create_note(guard: Guard, noteform: Form<NoteForm>, api: &State<ApiClient>) -> Redirect {
    let note = noteform.into_inner();
    let _: Note = api
        .post("/notes/", guard.access_token(), None, &note)
        .await
        .unwrap();
    let redirect = format!("/{}/{}", note.category, note.parent);
    Redirect::to(redirect)
}

#[get("/create?<parent>&<category>")]
async fn create_note_html(parent: Uuid, category: String) -> RawHtml<Template> {
    RawHtml(Template::render(
        "notes/create",
        context! { title: "create note", parent, category },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![create_note, create_note_html]
}
