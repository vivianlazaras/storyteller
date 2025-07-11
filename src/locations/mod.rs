use crate::ApiClient;
use crate::assets::images::ImageForm;
use crate::auth::Guard;
use crate::model::{Location, Tag};
use rocket::fs::TempFile;
use crate::assets::images::{ImageBuilder, ImageProcessor};
use rocket::{
    FromForm, Route, State, delete, form::Form, get, post, put, response::Redirect,
    response::content::RawHtml, routes,
};
use crate::model::Image;
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRender {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub images: Option<Vec<Image>>,
    pub thumbnail: Option<Image>,
    pub tags: Option<Vec<Tag>>,
    pub created: Option<i64>,
}

#[derive(Debug, FromForm)]
pub struct LocationForm<'r> {
    name: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    images: Option<Vec<TempFile<'r>>>,
    imagetags: Option<Vec<String>>,
}

impl<'r> LocationForm<'r> {
    pub async fn to_builder(&self, processor: &ImageProcessor) -> anyhow::Result<LocationBuilder> {
        Ok(LocationBuilder {
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.as_ref().unwrap_or(&Vec::new()).to_vec(),
            thumbnail: self.into_image_builder(processor).await?,
        })
    }
}

impl<'r> ImageForm<'r> for LocationForm<'r> {
    fn images(&self) -> Option<&Vec<TempFile<'r>>> {
        self.images.as_ref()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tags(&self) -> &[String] {
        self.tags.as_deref().unwrap_or(&[])
    }

    fn category(&self) -> &str {
        "locations"
    }

    fn parent(&self) -> Option<Uuid> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationBuilder {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    thumbnail: Option<ImageBuilder>,
}

#[get("/")]
async fn list_places(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    let locations: Vec<LocationRender> = match api
        .get_protected("/locations/", guard.access_token(), None)
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
async fn get_place(guard: Guard, api: &State<ApiClient>, id: Uuid) -> RawHtml<Template> {
    let url = format!("/locations/{}", id);
    let location: LocationRender = api
        .get_protected(&url, guard.access_token(), None)
        .await
        .unwrap();
    println!("location: {:?}", location);
    RawHtml(Template::render(
        "locations/location",
        context! {title: location.name.clone(), location },
    ))
}

#[get("/create")]
async fn create_place_html(api: &State<ApiClient>) -> RawHtml<Template> {
    let options = api.get_top_tags(10, 0).await.unwrap();
    RawHtml(Template::render(
        "locations/create",
        context! { title: "create a setting", options },
    ))
}

#[post("/", data = "<form>")]
async fn create_place<'r>(
    guard: Guard,
    api: &State<ApiClient>,
    form: Form<LocationForm<'r>>,
    processor: &State<ImageProcessor>,
) -> Redirect {
    let locationform = form.into_inner();
    let location = locationform.to_builder(processor).await.unwrap();
    let loc: Location = api
        .post("/locations/", guard.access_token(), None, &location)
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
