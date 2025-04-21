use crate::{Record, AccessLevel};
use crate::loader::Sled;
use rocket::State;
use rocket::response::content::RawHtml;
use rocket::{Route, get, routes};
use rocket_dyn_templates::{Template, context};
use sled::Tree;
use sled::IVec;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: Uuid,
    #[serde(flatten)]
    pub record: Record,
    pub url: String,
    pub image_url: Option<String>,
}

impl Resource {
    pub fn access(&self) -> AccessLevel {
        self.record.owner.access
    }
}

#[get("/<id>")]
async fn get_resource(sled: &State<Sled>, id: Uuid) -> RawHtml<Template> {
    unimplemented!();
}

#[get("/")]
async fn list_resources(sled: &State<Sled>) -> RawHtml<Template> {
    let items: Vec<(IVec, IVec)> = sled.resources.iter().collect::<Result<_, _>>().unwrap();
    let resources = items
        .par_iter()
        .map(|(key, val)| serde_json::from_slice::<Resource>(&val))
        .filter(|r| match r {
            Ok(val) => val.access() == AccessLevel::Public,
            Err(_) => false,
        })
        .collect::<Result<Vec<Resource>, _>>()
        .unwrap();
    RawHtml(Template::render(
        "resources",
        context! { title: "public projects", resources: resources },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_resource, list_resources]
}
