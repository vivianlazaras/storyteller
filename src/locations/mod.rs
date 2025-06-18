use crate::ApiClient;
use crate::assets::images::ImageForm;
use crate::auth::Guard;
use crate::get_access_token;
use crate::model::Location;
use rocket::fs::TempFile;
use rocket::http::CookieJar;

use rocket::{
    FromForm, Route, State, delete, form::Form, get, post, put, response::Redirect,
    response::content::RawHtml, routes,
};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRender {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: String,
}

impl Location {
    pub fn render(self) -> LocationRender {
        LocationRender {
            id: self.id,
            name: self.name,
            description: self.description,
            created: crate::epoch_to_human(self.created),
        }
    }
}

#[derive(Debug, FromForm)]
pub struct LocationForm<'r> {
    name: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    images: Option<Vec<TempFile<'r>>>,
    imagetags: Option<Vec<String>>,
    category: String,
}

impl<'r> LocationForm<'r> {
    pub fn to_builder(&self) -> LocationBuilder {
        LocationBuilder {
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.as_ref().unwrap_or(&Vec::new()).to_vec(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationBuilder {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
}

#[get("/")]
async fn list_places(
    guard: Guard,
    api: &State<ApiClient>,
    jar: &CookieJar<'_>,
) -> RawHtml<Template> {
    let locations: Vec<Location> = match api
        .get_protected("/locations/", &get_access_token(jar), None)
        .await
        .unwrap()
    {
        Some(locations) => locations,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "locations/index",
        context! { title: "settings", locations },
    ))
}

#[get("/<id>")]
async fn get_place(
    guard: Guard,
    api: &State<ApiClient>,
    id: Uuid,
    jar: &CookieJar<'_>,
) -> RawHtml<Template> {
    let url = format!("/locations/{}", id);
    let location: Location = api
        .get_protected(&url, &get_access_token(jar), None)
        .await
        .unwrap();
    let render = location.render();
    RawHtml(Template::render(
        "locations/location",
        context! {title: render.name.clone(), location: render },
    ))
}

#[get("/create")]
async fn create_place_html() -> RawHtml<Template> {
    RawHtml(Template::render(
        "locations/create",
        context! { title: "create a setting" },
    ))
}

#[post("/", data = "<form>")]
async fn create_place<'r>(
    api: &State<ApiClient>,
    form: Form<LocationForm<'r>>,
    jar: &CookieJar<'_>,
) -> Redirect {
    let locationform = form.into_inner();
    let location = locationform.to_builder();
    let loc: Location = api
        .post("/locations/", &get_access_token(jar), None, &location)
        .await
        .unwrap();
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
