
use serde::{ser::Serialize, de::Deserialize};
use comrak::{Options, markdown_to_html};
use crate::*;
use crate::model::{StoryFragment, Story};
use rocket::form::Form;
use rocket::http::CookieJar;
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
    renderer: SupportedRender,
    content: String,
}

#[get("/<id>")]
async fn get_story(id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    //api.get("/stories/")
    unimplemented!();
}

#[get("/")]
async fn list_stories(api: &State<ApiClient>) -> RawHtml<Template> {
    //let user =;
    //let stories = StoryFragment::belonging_to(&user)
    let resp: Vec<Story> = api.get("/stories", None).await.unwrap();
    RawHtml(
        Template::render("stories/index", context! { title: "published stories", stories: resp })
    )
}


#[get("/create")]
async fn create_story_html() -> RawHtml<Template> {
    RawHtml(
        Template::render("stories/create", context! { title: "create story" })
    )
}

#[post("/", data="<story>")]
async fn create_story(user: Guard, jar: &CookieJar<'_>, auth: &State<rocket_oidc::AuthState>, story: Form<CreateStoryFragment>, api: &State<ApiClient>) -> Redirect {
    let story = story.into_inner();
    let access_token = jar.get("access_token").unwrap().to_string();
    //println!("create story called: {}", serde_json::to_string(&story).unwrap());
    //let token_response = auth.client.exchange_token_for_audience(&access_token, "storyteller-api").await.unwrap();

    api.post("/stories", &access_token, story).await.unwrap();
    Redirect::to("/")
}

pub struct Edit {
    id: Uuid,
    date: i64,
    comment: Option<String>,
    addition: bool,
    editor: Uuid,
    value: String
}

pub struct StoryEdit {
    edits: Vec<Edit>,
}

#[put("/")]
async fn edit_story() {

}

pub fn get_routes() -> Vec<Route> {
    routes![list_stories, create_story, create_story_html, get_story]
}