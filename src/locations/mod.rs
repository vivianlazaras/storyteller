use crate::ApiClient;
use rocket::{Route, response::Redirect, form::Form, FromForm, State, delete, get, post, put, response::content::RawHtml, routes};
use rocket_dyn_templates::{Template, context};

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRender {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: String,
}


impl Location {
    pub fn render(self) -> LocationRender {
        LocationRender { id: self.id, name: self.name, description: self.description, created: crate::epoch_to_human(self.created) }
    }
}

#[derive(Debug, FromForm, Serialize)]
pub struct LocationForm {
    name: String,
    description: Option<String>,
}

#[get("/")]
async fn list_places(api: &State<ApiClient>) -> RawHtml<Template> {
    let locations: Vec<Location> = match api.get("/locations/", None).await.unwrap() {
        Some(locations) => locations,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "locations/index",
        context! { title: "settings", locations },
    ))
}

#[get("/<id>")]
async fn get_place(api: &State<ApiClient>, id: Uuid) -> RawHtml<Template> {
    let url = format!("/locations/{}", id);
    let location: Location = api.get(&url, None).await.unwrap();
    let render = location.render();
    RawHtml(
        Template::render("locations/location", context! {title: render.name.clone(), location: render })
    )
}

#[get("/create")]
async fn create_place_html() -> RawHtml<Template> {
    RawHtml(Template::render(
        "locations/create",
        context! { title: "create a setting" },
    ))
}

#[post("/", data="<form>")]
async fn create_place(api: &State<ApiClient>, form: Form<LocationForm>) -> Redirect {
    let location = form.into_inner();
    let loc: Location = api.post("/locations/", "", None, &location).await.unwrap();
    Redirect::to(format!("/locations/{}", loc.id))
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
