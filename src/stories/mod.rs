use crate::api::{ApiClient, get_access_token};
use crate::auth::Guard;
use crate::characters::api::CharacterRender;
use crate::model::Task;
use crate::model::{Character, Story, StoryFragment, Tag};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::{FromForm, FromFormField};
use std::collections::HashMap;

use rocket::{
    Route, State, delete, get, post, put,
    response::{Redirect, content::RawHtml},
    routes,
};

use crate::locations::LocationRender;

use crate::render::SupportedRender;
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountBtn {
    pub text: &'static str,
    pub link: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct StoryBuilder {
    title: String,
    description: Option<String>,
    renderer: SupportedRender,
    #[field(name = "tags")]
    pub tags: Vec<String>,
    /// The group in which this story is created in, will default to user's default group
    pub group: Option<Uuid>,
}

#[get("/<id>")]
async fn get_story(
    guard: Guard,
    id: Uuid,
    api: &State<ApiClient>,
    jar: &CookieJar<'_>,
) -> RawHtml<Template> {
    let access_token = get_access_token(jar);
    let url = format!("/stories/{}", id);
    let id_string = id.to_string();
    let story: Story = api.get_protected(&url, &access_token, None).await.unwrap();
    let mut params = HashMap::new();
    params.insert("parent", id_string.as_str());
    let tagurl = format!("/tags/{}", story.id);
    let tags: Vec<Tag> = api.get(&tagurl, None).await.unwrap();

    let fragments = match api
        .get_protected::<Option<Vec<StoryFragment>>, _>(
            "/fragments",
            &access_token,
            Some(params.clone()),
        )
        .await
        .unwrap()
    {
        Some(fragments) => Some(
            fragments
                .into_iter()
                .map(|f| f.render())
                .collect::<Vec<crate::fragments::frontend::FragmentRender>>(),
        ),
        None => None,
    };

    /// may need to be implemented later, for now don't worry about it
    /// I have to grab each character for each fragment and assembly them.
    let characters: Option<Vec<CharacterRender>> = match api
        .get_protected::<Option<Vec<CharacterRender>>, _>(
            "/characters/filter",
            &access_token,
            Some(params.clone()),
        )
        .await
        .unwrap()
    {
        Some(characters) => Some(characters),
        None => None,
    };

    let locations: Option<Vec<LocationRender>> = api.get_protected("/locations/filter", &get_access_token(jar), Some(params.clone())).await.unwrap();

    let notes: Option<Vec<Task>> = api
        .get_protected("/notes/", &access_token, Some(params))
        .await
        .unwrap();
    RawHtml(Template::render(
        "stories/story",
        context! { title: story.name.clone(), notes, story, fragments, characters, tags, locations },
    ))
}

#[get("/")]
async fn list_stories(
    guard: Guard,
    api: &State<ApiClient>,
    cookies: &CookieJar<'_>,
) -> RawHtml<Template> {
    //let access_token = cookies.get("access_token").unwrap();
    let resp: Vec<Story> = api
        .get_protected("/stories", &get_access_token(&cookies), None)
        .await
        .unwrap();
    RawHtml(Template::render(
        "stories/index",
        context! { title: "published stories", stories: resp },
    ))
}

#[get("/create")]
async fn create_story_html(api: &State<ApiClient>) -> RawHtml<Template> {
    let selected: Vec<String> = Vec::new();
    let options = api.get_top_tags(10, 0).await.unwrap();
    RawHtml(Template::render(
        "stories/create",
        context! { title: "create story", selected, options },
    ))
}

#[post("/", data = "<story>")]
async fn create_story(
    guard: Guard,
    jar: &CookieJar<'_>,
    //auth: &State<rocket_oidc::AuthState>,
    story: Form<StoryBuilder>,
    api: &State<ApiClient>,
) -> Redirect {
    let story = story.into_inner();
    let result: Story = api
        .post("/stories", &get_access_token(jar), None, &story)
        .await
        .unwrap();

    Redirect::to(format!("/stories/{}", result.id))
}

pub struct Edit {
    pub id: Uuid,
    pub date: i64,
    pub comment: Option<String>,
    pub addition: bool,
    pub editor: Uuid,
    pub value: String,
}

pub struct StoryEdit {
    edits: Vec<Edit>,
}

#[put("/<id>")]
async fn edit_story(id: Uuid) {
    unimplemented!();
}

#[delete("/<id>")]
async fn delete_story(id: Uuid, api: &State<ApiClient>) -> Redirect {
    api.delete("/stories/", id).await.unwrap();
    Redirect::to("/stories")
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
        delete_story
    ]
}
