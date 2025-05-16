
use serde::{ser::Serialize, de::Deserialize};
use comrak::{Options, markdown_to_html};
use crate::*;
use crate::model::StoryFragment;
use rocket::form::Form;
use rocket::{
    Route, State, get, post, put,
    response::{Redirect, content::RawHtml},
    routes
};

use crate::users::Guard;
use crate::render::SupportedRender;
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

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct CreateStoryFragment {
    name: String,
    description: Option<String>,
    render: SupportedRender,
    content: String,
}

#[get("/")]
async fn list_stories(user: Guard) -> RawHtml<Template> {
    //let user =;
    //let stories = StoryFragment::belonging_to(&user)
    unimplemented!();
}


#[get("/create")]
async fn create_story_html(user: Guard) -> RawHtml<Template> {
    RawHtml(
        Template::render("stories/create", context! {})
    )
}

#[post("/", data="<story>")]
async fn create_story(user: Guard, cookies: &CookieJar<'_>, story: Form<CreateStoryFragment>) -> Redirect {
    // either successful creation, or failure which results in redirect back
    let access_token = cookies.get("access_token");
    
    let res = client
        .get(api_url)
        .bearer_auth(access_token)  // adds Authorization: Bearer <token>
        .send()
        .await
        .unwrap();
    
    Redirect::to("/")
}

pub fn get_routes() -> Vec<Route> {
    routes![list_stories, create_story, create_story_html]
}