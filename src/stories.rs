use crate::model::{Character, Story, StoryFragment};
use crate::*;
use comrak::{Options, markdown_to_html};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::{
    Route, State, delete, get, post, put,
    response::{Redirect, content::RawHtml},
    routes,
};
use serde::{de::Deserialize, ser::Serialize};

use crate::render::SupportedRender;
use crate::users::Guard;
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
    let url = format!("/stories/{}", id);
    let id_string = id.to_string();
    let story: Story = api.get(&url, None).await.unwrap();
    let mut params = HashMap::new();
    params.insert("story", id_string.as_str());
    let fragments: Option<Vec<StoryFragment>> = api
        .get("/stories/fragments", Some(params.clone()))
        .await
        .unwrap();

    /// may need to be implemented later, for now don't worry about it
    /// I have to grab each character for each fragment and assembly them.
    // let characters: Option<Vec<Character>> = api.get("/characters/filter", Some(params)).await.unwrap();
    let characters: Vec<Character> = Vec::new();
    RawHtml(Template::render(
        "stories/story",
        context! { title: story.name.clone(), story, fragments, characters },
    ))
}

#[get("/")]
async fn list_stories(api: &State<ApiClient>) -> RawHtml<Template> {
    //let user =;
    //let stories = StoryFragment::belonging_to(&user)
    let resp: Vec<Story> = api.get("/stories", None).await.unwrap();
    RawHtml(Template::render(
        "stories/index",
        context! { title: "published stories", stories: resp },
    ))
}

#[get("/create")]
async fn create_story_html() -> RawHtml<Template> {
    RawHtml(Template::render(
        "stories/create",
        context! { title: "create story" },
    ))
}

// id and category can be used to generate a redirect, and link automatically
#[get("/fragments/create?<id>&<category>")]
async fn create_fragment_html(id: Option<Uuid>, category: Option<String>) -> RawHtml<Template> {
    // id (the entity to link with)
    // category (the type of entity)
    let mut redirect = None;
    if let Some(id) = id {
        if let Some(category) = category {
            redirect = Some(format!("?{}&{}", category, id));
        }
    }
    RawHtml(Template::render(
        "stories/fragments/create",
        context! { title: "create new fragment", redirect },
    ))
}

#[get("/fragments/<id>")]
async fn get_fragment(id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/stories/fragments/{}", id);
    let fragment: StoryFragment = api.get(&url, None).await.unwrap();
    RawHtml(Template::render(
        "stories/fragment",
        context! { title: fragment.name.clone(), fragment },
    ))
}

#[post("/", data = "<story>")]
async fn create_story(
    //guard: Guard,
    //jar: &CookieJar<'_>,
    //auth: &State<rocket_oidc::AuthState>,
    story: Form<CreateStoryFragment>,
    api: &State<ApiClient>,
) -> Redirect {
    let story = story.into_inner();
    //let access_token = jar.get("access_token").unwrap().to_string();
    //println!("create story called: {}", serde_json::to_string(&story).unwrap());
    //let token_response = auth.client.exchange_token_for_audience(&access_token, "storyteller-api").await.unwrap();

    api.post("/stories", "", story).await.unwrap();
    Redirect::to("/")
}

pub struct Edit {
    id: Uuid,
    date: i64,
    comment: Option<String>,
    addition: bool,
    editor: Uuid,
    value: String,
}

pub struct StoryEdit {
    edits: Vec<Edit>,
}

#[put("/<id>")]
async fn edit_story(id: Uuid) {
    unimplemented!();
}

#[delete("/<id>")]
async fn delete_story(id: Uuid) {
    unimplemented!();
}

/*#[post("/search", data = "<term>")]
async fn search(term: Form<>, api: &State<ApiClient>) -> RawHtml<Template> {

}*/

pub fn get_routes() -> Vec<Route> {
    routes![
        list_stories,
        create_story,
        create_story_html,
        get_story,
        edit_story,
        delete_story,
        get_fragment,
        create_fragment_html
    ]
}
