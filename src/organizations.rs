use uuid::Uuid;
use rocket::{get, post, routes, Route, State};
use crate::api::{ApiClient};
use crate::auth::Guard;

pub struct Organization {
    id: Uuid,
    name: String,
    description: Option<String>,
}

pub struct OrganizationRender {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: u64,
    last_edited: Option<u64>,

}

#[post("/")]
fn create_organization(guard: Guard) {}

#[get("/<id>")]
fn get_organization(guard: Guard, id: Uuid, api: &State<ApiClient>) {}

#[get("/")]
fn list_organizations(guard: Guard) {}

pub fn get_routes() -> Vec<Route> {
    routes![create_organization, get_organization, list_organizations]
}