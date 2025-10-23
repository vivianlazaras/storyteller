use crate::ApiClient;
use crate::auth::Guard;
use crate::fragments::api::FragmentBuilder;
use crate::relations::RelatedEntity;
use crate::timelines::api::Timeline;
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, form::Form, get, post, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct TimelineForm {
    pub name: String,
    pub description: Option<String>,
    pub source: Uuid,
    pub category: String,
}

#[get("/<id>")]
async fn get_timeline(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/timelines/{}", id);
    let timeline: Timeline = api
        .get_protected(&url, guard.access_token(), None)
        .await
        .unwrap();

    let render = timeline.render();
    RawHtml(Template::render(
        "timelines/timeline",
        context! { title: render.name.clone(), timeline: render },
    ))
}

#[get("/")]
async fn list_timelines(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    let timelines: Vec<RelatedEntity> = match api
        .get_protected("/timelines", guard.access_token(), None)
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

#[get("/create")]
async fn create_html(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    RawHtml(Template::render(
        "timelines/create",
        context! { title: "create new timeline" },
    ))
}

#[post("/", data = "<form>")]
async fn create_timeline(
    guard: Guard,
    api: &State<ApiClient>,
    form: Form<TimelineForm>,
) -> Redirect {
    let form = form.into_inner();
    let timeline: Timeline = api
        .post("/timelines", guard.access_token(), None, &form)
        .await
        .unwrap();
    Redirect::to(format!("/{}/{}", form.category, form.source))
}
pub fn get_routes() -> Vec<Route> {
    routes![get_timeline, list_timelines, create_html, create_timeline]
}
