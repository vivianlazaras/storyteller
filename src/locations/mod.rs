use crate::ApiClient;
use rocket::{Route, State, delete, get, post, put, response::content::RawHtml, routes};
use rocket_dyn_templates::{Template, context};

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    id: Uuid,
    name: String,
    description: Option<String>,
}

#[get("/")]
async fn list_places(api: &State<ApiClient>) -> RawHtml<Template> {
    let places: Vec<Place> = match api.get("/locations/", None).await.unwrap() {
        Some(places) => places,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "locations/index",
        context! { title: "settings", places },
    ))
}

#[get("/<id>")]
async fn get_place(api: &State<ApiClient>, id: Uuid) -> RawHtml<Template> {
    unimplemented!();
}

#[get("/create")]
async fn create_place_html() -> RawHtml<Template> {
    RawHtml(Template::render(
        "locations/create",
        context! { title: "create a setting" },
    ))
}

#[post("/")]
async fn create_place() {
    unimplemented!();
}

#[put("/<id>")]
async fn update_place(id: Uuid) {
    unimplemented!();
}

#[delete("/<id>")]
async fn delete_place(id: Uuid) {
    unimplemented!();
}

pub fn get_routes() -> Vec<Route> {
    routes![
        list_places,
        get_place,
        create_place_html,
        create_place,
        update_place,
        delete_place
    ]
}
