use crate::loader::Sled;
use crate::{AccessLevel, Ownership, Record};
use rayon::prelude::*;
use rocket::form::Form;
use rocket::response::{Redirect, content::RawHtml};
use rocket::{FromForm, Route, State, get, post, routes};
use sled::IVec;
use uuid::Uuid;

use rocket_dyn_templates::{Template, context};
use rocket_oidc::{CoreClaims, OIDCGuard};
use sled::Tree;

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectGuard {
    sub: String,
    email: String,
}

impl CoreClaims for ProjectGuard {
    fn subject(&self) -> &str {
        self.sub.as_str()
    }
}

type Guard = OIDCGuard<ProjectGuard>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    id: Uuid,
    #[serde(flatten)]
    pub record: Record,
    pub version: String,
    pub image_url: Option<String>,
    /// the projects that need to be completed before this project can be completed
    pub dependencies: Vec<Uuid>,
    /// url to source code, such as git url
    pub source: Option<String>,
}

impl Project {
    pub fn access(&self) -> AccessLevel {
        self.record.owner.access
    }
}

#[derive(Serialize, FromForm, Deserialize, Debug, Clone)]
pub struct CreateProject {
    /// for now I'm going to expand the fields of Record until I can find a better solution
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub image_url: String,
    pub source: Option<String>,
    pub dependencies: Vec<String>,
}

#[get("/<id>")]
async fn get_project(tree: &State<Sled>, id: Uuid) -> RawHtml<Template> {
    unimplemented!();
}

// this should list only public projects, but access levels will be implemented later
#[get("/")]
async fn list_pub_projects(sled: &State<Sled>) -> RawHtml<Template> {
    let items: Vec<(IVec, IVec)> = sled.projects.iter().collect::<Result<_, _>>().unwrap();
    let projects = items
        .par_iter()
        .map(|(key, val)| serde_json::from_slice::<Project>(&val))
        .filter(|r| match r {
            Ok(val) => val.access() == AccessLevel::Public,
            Err(_) => false,
        })
        .collect::<Result<Vec<Project>, _>>()
        .unwrap();
    RawHtml(Template::render(
        "projects",
        context! { title: "public projects", projects: projects },
    ))
}

#[get("/")]
async fn list_projects(guard: Guard) -> RawHtml<Template> {
    unimplemented!();
}

#[post("/create", data = "<project>")]
async fn create_project(
    guard: Guard,
    tree: &State<Sled>,
    project: Form<CreateProject>,
) -> Redirect {
    unimplemented!();
}

pub fn get_routes() -> Vec<Route> {
    routes![get_project, list_pub_projects, create_project]
}
