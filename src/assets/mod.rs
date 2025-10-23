pub mod audio;
pub mod datasets;
pub mod exif;
pub mod graphs;
pub mod images;
pub mod models;

use rocket::{Route, get, response::content::RawHtml, routes};
use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn index() -> RawHtml<Template> {
    RawHtml(Template::render(
        "assets/index",
        context! { title: "Asset Main Page" },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
