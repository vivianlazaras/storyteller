use crate::ApiClient;
use crate::auth::Guard;
use crate::fragments::api::FragmentBuilder;
use crate::get_access_token;
use crate::links::RelatedEntity;
use crate::timelines::api::Timeline;
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::response::content::RawHtml;
use rocket::{Route, State, get, post, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

pub struct TimelineForm<'r> {
    pub name: String,
    pub description: Option<String>,
    /// This is intended as a means for defining a SVG superset for representing timelines
    /// accross platforms where each node contains either UUID to reference an entity, or this projects
    /// JSON spec to represent first class entities along a timeline.
    /// however processing this entity is currently not implemented, thus does not show up in the UI.
    pub svg: Option<TempFile<'r>>,
}

#[get("/<id>")]
async fn get_timeline(
    guard: Guard,
    id: Uuid,
    api: &State<ApiClient>,
    jar: &CookieJar<'_>,
) -> RawHtml<Template> {
    let url = format!("/timelines/{}", id);
    let timeline: Timeline = api
        .get_protected(&url, &get_access_token(jar), None)
        .await
        .unwrap();
    println!("timeline: {}", serde_json::to_string(&timeline).unwrap());
    let render = timeline.render();
    //println!("render: {:?}", render);
    RawHtml(Template::render(
        "timelines/timeline",
        context! { title: render.name.clone(), timeline: render },
    ))
}

#[get("/")]
async fn list_timelines(
    guard: Guard,
    api: &State<ApiClient>,
    jar: &CookieJar<'_>,
) -> RawHtml<Template> {
    let timelines: Vec<RelatedEntity> = match api
        .get_protected("/timelines", &get_access_token(jar), None)
        .await
        .unwrap()
    {
        Some(entities) => entities,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "timelines/index",
        context! { title: "timelines", timelines },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_timeline, list_timelines]
}
