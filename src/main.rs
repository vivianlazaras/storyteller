#[macro_use]
extern crate rocket;
use qrespite::stories::{AccountBtn, Story, StoryTitle};
use rocket::State;
use rocket::fs::FileServer;
use rocket::response::{Redirect, content::RawHtml};
use rocket_dyn_templates::{Template, context};
use rocket_oidc::OIDCConfig;
use sled::Tree;
use std::str::FromStr;
use uuid::Uuid;

#[catch(401)]
fn unauthorized() -> Redirect {
    Redirect::to("/")
}

#[get("/")]
async fn index() -> RawHtml<Template> {
    RawHtml(Template::render(
        "index",
        context! { title: "Queer Respite" },
    ))
}

#[launch]
async fn rocket() -> _ {
    let story_owner_id = Uuid::from_str(&std::env::var("STORY_OWNER_ID").unwrap()).unwrap();
    let sled = qrespite::loader::Sled::load("../", "records.db", story_owner_id)
        .await
        .unwrap();

    let mut rocket = rocket::build()
        .mount("/", routes![index])
        .mount("/stories", qrespite::stories::get_routes())
        .register("/", catchers![unauthorized])
        .attach(Template::fairing())
        .manage(sled)
        .mount("/static", FileServer::from("static"))
        .mount("/projects", qrespite::projects::get_routes())
        .mount("/resources", qrespite::resources::get_routes())
        .mount("/users", qrespite::users::get_routes());
    rocket_oidc::setup(rocket, OIDCConfig::from_env().unwrap())
        .await
        .unwrap()
}
