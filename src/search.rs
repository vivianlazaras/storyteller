use crate::ApiClient;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, get, post, routes};
use rocket_dyn_templates::{Template, context};
use std::collections::HashMap;

pub struct SearchCriteria {
    name: Option<String>,
    tags: Option<Vec<String>>,
}

impl SearchCriteria {
    pub fn with_name(name: String) -> Self {
        Self {
            name: Some(name),
            tags: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Character,
    Story,
    Place,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
pub struct TimeRange {
    start: u64,
    end: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TagCount {
    value: String,
    count: i32,
}

#[get("/advanced/<category>")]
async fn advanced_search_html(category: String, api: &State<ApiClient>) -> RawHtml<Template> {
    let selected: Vec<String> = Vec::new();
    let mut params = HashMap::new();
    params.insert("limit", "10");
    params.insert("min_count", "0");

    let options_opt: Option<Vec<TagCount>> = api
        .get("/analytics/populartags", Some(params))
        .await
        .unwrap();
    let options = match options_opt {
        Some(options) => options,
        None => Vec::new(),
    };
    // fetch most popular tags
    RawHtml(Template::render(
        "search/advanced",
        context! { title: "advanced search", category: category, selected, options },
    ))
}

/*#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct AdvancedSearch {
    name: Option<String>,
    tags: Vec<String>,
    category: Option<Category>,
    //created: Option<TimeRange>,
    //last_edited: Option<TimeRange>,
    owner: Option<String>,
}

#[post("/search/advanced")]
async fn advanced_search() -> RawHtml<Template> {
    unimplemented!();
}*/

pub fn get_routes() -> Vec<Route> {
    routes![advanced_search_html]
}
