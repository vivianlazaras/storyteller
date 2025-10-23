//! This module functions to provide sharing for entities.
//!
//! # Supported Operations
//! 1. Add Member
//! 2. Remove Member
//! 3. Create Group
//! 4. Delete Group
//! 5. List Groups/Show Group
//! 6. Hide Group

use crate::ApiClient;
use crate::auth::Guard;
use crate::errors::ApiError;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, form::Form, get, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GroupBuilder {
    pub name: String,
    pub hidden: bool,
    pub parent_id: Option<Uuid>,
    pub user_ids: Vec<Uuid>,
}

impl GroupBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hidden: false,
            parent_id: None,
            user_ids: Vec::new(),
        }
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRender {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub permissions: Option<Vec<String>>,
    pub users: Option<Vec<Uuid>>,
    pub child_groups: Option<Vec<Uuid>>,
    
}

/*
#[post("/", data = "<form>")]
async fn create_group(
    guard: Guard,
    api: &State<ApiClient>,
    form: Form<GroupBuilder>,
) -> Result<RawHtml<String>> {
    let builder = form.into_inner();

    // Build the POST request
    let req = api
        .request("/groups")             // Base route to create group
        .method(Method::POST)
        .access_token(guard.token())   // Pass token from guard
        .params(builder.into_params()) // convert GroupBuilder -> HashMap<String, String> or Map
        ;

    // Send the request (assuming ApiRequest has async send method)
    let response = req.send().await?;

    // Handle response, for example rendering HTML or JSON
    let body = response.text().await?;
    Ok(RawHtml(body))
}

#[get("/<id>")]
async fn get_group(
    guard: Guard,
    api: &State<ApiClient>,
    id: Uuid,
) -> Result<RawHtml<Template>> {
    let route = format!("/groups/{}", id);

    let req = api
        .request(&route)
        .access_token(guard.access_token());

    let response = req.send().await?;
    let json = response.json::<Group>().await?;

    // Render your template with the group data
    Ok(RawHtml(Template::render("group/show", &json)))
}*/

#[get("/")]
async fn list_groups(guard: Guard, api: &State<ApiClient>) -> Result<RawHtml<Template>, ApiError> {
    let req = api.request("/groups").access_token(guard.access_token());

    let groups: Vec<GroupRender> = req.send().await?;
    //let groups = response.json::<Vec<GroupRender>>().await?;

    Ok(RawHtml(Template::render("group/index", &groups)))
}

pub fn get_routes() -> Vec<Route> {
    //routes![create_group, get_group, list_groups]
    Vec::new()
}
