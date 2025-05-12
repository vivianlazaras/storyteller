
use serde::{ser::Serialize, de::Deserialize};
use comrak::{Options, markdown_to_html};
use rocket::{
    Route, State, get,
    response::{Redirect, content::RawHtml},
    routes,
};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountBtn {
    pub text: &'static str,
    pub link: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryTitle {
    pub id: Uuid,
    pub name: String,
}

#[get("/")]
pub async fn list_stories()

pub fn get_routes() -> Vec<Route> {
    routes![]
}
